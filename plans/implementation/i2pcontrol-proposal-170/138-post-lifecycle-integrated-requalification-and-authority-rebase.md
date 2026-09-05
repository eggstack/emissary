# M138 — Post-Lifecycle Integrated Requalification and Authority Rebase

Status: **ready / registered**

Class: invariant / qualification / roadmap-correction

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Planning baseline:

- current `master`: `e4f217cb1459e26bf011da46b67fc2c83cd192b5`;
- M134 closure: `plans/closure/i2pcontrol-proposal-170/134-closure.md`;
- M135 closure: `plans/closure/i2pcontrol-proposal-170/135-closure.md`;
- M136 closure: `plans/closure/i2pcontrol-proposal-170/136-closure.md`;
- M137 closure: `plans/closure/i2pcontrol-proposal-170/137-closure.md`;
- current M095 authority: `325 apply / 47 blocked_primitive / 468 not_applicable`;
- M130 remains the historical implemented-subset runtime/security qualification authority pending this milestone;
- M131 remains residual applicability/primitive authority for the remaining non-lifecycle clusters.

Pinned Proposal authority:

- I2P Proposal 170 revision `2026-05-20`, status Open;
- pinned Proposal SHA-256 remains the M095 authority.

Architecture/security authority:

- canonical plans `000`–`003`;
- ADR-0001 through ADR-0005;
- M061/M062 containment and dependency ownership;
- M093 tunnel security;
- M110/M116 shared-session/destination ownership;
- M123 cancellation/commit atomicity;
- M127 finite token lifetime;
- M128 bounded JSON-RPC batches;
- M129 fail-closed non-loopback TLS;
- M130 historical integrated requalification;
- M131 residual applicability/primitive map;
- M134/M135/M136/M137 lifecycle closure chain.

External/upstream repositories and specifications are read-only evidence. All writes authorized by this plan remain internal to `eggstack/emissary`.

## 1. Objective

Requalify the complete **currently implemented** Proposal-170 subset after the M135→M136→M137→M134 lifecycle implementation chain, remove stale current-head test/document assertions inherited from M126/M130, and establish one new current-head runtime/security qualification authority.

M138 is successful only if the repository can truthfully say:

1. the current `325/47/468` matrix is mechanically consistent with production behavior and active documentation;
2. the M127–M130 shared-control-plane security guarantees still hold after lifecycle work;
3. M135 live quantity/LeaseSet control, M136 idle reduction, M137 idle close/reasoned teardown, and M134 proven-resume NewDest all compose without violating earlier containment/security/runtime invariants;
4. current-head regression guards no longer fail merely because they hard-code superseded pre-M131/pre-lifecycle baselines;
5. active roadmaps/registry/indexes consistently name the same current authority and residual count;
6. no high/medium correctness or security defect is discovered in the claimed implemented subset.

If these conditions hold, M138 closure becomes the new current implemented-subset runtime/security qualification authority and supersedes M130 **only for current-head qualification**. Historical M130 closure remains immutable evidence for its original baseline.

M138 is not a Proposal capability milestone. It must not increase support counts merely to improve completeness.

## 2. Hard dependencies and readiness

Hard dependencies are closed:

- M134 — NewDest on proven idle resume;
- M135 — neutral live tunnel quantity + dynamic LeaseSet desired count;
- M136 — canonical SAM activity/idle reduction + Proposal `Reduce*`;
- M137 — canonical idle close/reasoned termination + Proposal `Close*`.

Current evidence making M138 dependency-ready:

- `plans/registry.md` and M095 agree on `325/47/468`;
- M134 closure records the lifecycle roadmap complete;
- M134 closure reports core, I2PControl library, containment/matrix and live-runtime evidence green on the current implementation line;
- M136/M137/M134 closures explicitly record stale historical/current-head tests that still assert older baselines;
- `emissary-cli/tests/m126_requalification.rs` still hard-codes `284/96/460` and labels that as current authority;
- `emissary-cli/tests/m130_post_corrective_requalification.rs` still hard-codes `284/96/460` and contains a baseline-specific assertion that M130 changed no core/composition paths, which is historically valid for M130 but invalid as a current-head assertion after M135–M137;
- the full-support roadmap header is current, while stale body text still describes `284/88/468` and 88 blocked residuals.

This is therefore qualification/authority drift with a bounded correction surface, not an unresolved architecture dependency.

## 3. Invariants

M138 preserves all of the following.

### 3.1 Proposal/support truthfulness

- current matrix starts at exactly `325 apply / 47 blocked_primitive / 468 not_applicable`;
- every `apply` cell must correspond to real runtime behavior;
- unsupported supplied values continue to fail before allocation/effect;
- `not_applicable` cells remain evidence-backed, not convenience demotions;
- full Proposal 170 support must not be claimed while any applicable blocker remains.

M138 itself has a **zero-promotion budget**. A matrix count change is not an implementation objective.

If requalification proves that an existing `apply` claim is wrong, do not paper over it. Record the discrepancy, demote only if the evidence requires it, and close M138 as corrective-required/blocked with a follow-up plan. A support demotion discovered by requalification is allowed only as a truthfulness correction, never as a shortcut to green tests.

### 3.2 Containment

- Proposal/admin policy remains under `emissary-cli/src/i2pcontrol/**`;
- existing neutral M135–M137 core seams remain the only lifecycle-related exceptions already accepted by their closures;
- M138 authorizes no new production path outside those accepted seams;
- M061/M062 must describe the actual current non-policy diff against the pinned upstream baseline without broad globs or silent allowances;
- a test baseline may be rebased, but a production containment rule may not be weakened merely because current code violates it.

### 3.3 Security/runtime invariants

Requalify, do not merely inherit:

- finite API token lifetime and reachable `TOKEN_EXPIRED`;
- bounded token/throttle state;
- bounded JSON-RPC body, request, connection-task and batch admission;
- per-element batch authentication and notification suppression;
- TLS-only dispatch;
- managed TLS loopback-only and explicit material required for non-loopback binds;
- validation-before-listener/filesystem side effects for rejected remote TLS configuration;
- no plaintext fallback;
- no secret/token/password/private destination leakage;
- no direct-clearnet fallback or local-target confinement weakening;
- bounded lifecycle timers/control cells/eligibility state;
- generation-local idle state and termination reasons;
- manual/restart/failure paths must not fabricate `IdlePolicy` or rotate NewDest;
- shared sessions retain one identity/activity/idle policy and one successor;
- exploratory/participating tunnel pools remain outside client quantity control;
- LeaseSets never advertise nonexistent tunnels;
- no lock spans unrelated network/filesystem I/O, joins, sleeps, or timer waits.

### 3.4 Historical evidence

Historical closure records are immutable.

Tests named for M126/M130 may continue to assert the behavior those milestones established, but they must not mislabel obsolete counts or obsolete path budgets as the current-head authority. Where a test mixes historical milestone facts with current-head assertions, split the concerns or rewrite the current-head portion to consume current machine authority.

## 4. Explicit non-goals

M138 must not:

- implement any of the remaining 47 blocked cells;
- start work on `UseSSL`, `SigType`, `UseOutproxyPlugin`, `SSLProxies`, `JumpList`, `Profile`, Streamr `ConnectDelay`, `UniqueLocalAddressPerClient`, `MultiHoming`/`shouldBundleReplyInfo`, or encrypted/authenticated LeaseSets;
- add a new tunnel type, action, RPC method, status, alias, or base-I2PControl parity method;
- redesign tunnel pools, SAM lifecycle, LeaseSet publication, secret storage, or shared-session ownership;
- modify Yosemite or dependency pins;
- change router algorithms, peer selection, transports, NetDb, crypto, or frontend behavior;
- normalize unrelated rustfmt drift;
- add hosted CI/release infrastructure;
- prepare or request upstream review/submission.

If a production defect is found that requires runtime code changes, M138 stops and records a corrective plan rather than expanding this qualification milestone into an implementation refactor.

## 5. Current drift inventory to reconcile

The implementation agent must begin by mechanically inventorying every current-head failure before editing tests.

Known starting drift includes at least:

1. `emissary-cli/tests/m126_requalification.rs`
   - hard-codes `284/96/460` as current matrix;
   - requires active documents to contain that superseded count.
2. `emissary-cli/tests/m130_post_corrective_requalification.rs`
   - hard-codes `284/96/460` as current matrix;
   - requires active documents to state that superseded count;
   - contains an M130-baseline containment check whose historical assertion is valid for the M130 diff but must not be applied as a current-head prohibition against the accepted M135–M137 core/composition seams.
3. `emissary-cli/tests/m060_containment.rs`
   - compares all core changes since the old upstream baseline against the historical M060 path budget and may reject later explicitly accepted M135–M137 neutral owner paths.
4. `emissary-cli/tests/m061_containment.rs`
   - remains current containment authority and must be checked against the updated `061-containment-boundary.toml`; any mismatch is a real governance problem, not automatically test drift.
5. `emissary-cli/tests/m062_dependency_containment.rs`
   - contains accumulated milestone path exceptions; obsolete current-status assumptions and any stale planning-path list must be reconciled without weakening dependency isolation.
6. M127/M128/M129 tests
   - should remain behavior/security regressions; inspect for baseline/path assertions that accidentally treat M130 as immutable current head.
7. active documentation
   - `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` has a current header but stale body references to `284/88/468` and 88 blockers;
   - `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md` still calls M130 current-head authority even though lifecycle production work landed later;
   - registry/index/AGENTS/docs must be checked for contradictory current authority/count language.

Do not assume this list is exhaustive. Produce a baseline failure/contradiction ledger before changes.

## 6. Required changes and path budget

M138 is planning/test/documentation work. **No production Rust source change is pre-authorized.**

Expected test/static-guard paths, only when evidence shows current-head drift:

- `emissary-cli/tests/m060_containment.rs`;
- `emissary-cli/tests/m061_containment.rs`;
- `emissary-cli/tests/m062_dependency_containment.rs`;
- `emissary-cli/tests/m095_full_support_matrix.rs`;
- `emissary-cli/tests/m105_residual_option_audit.rs`;
- `emissary-cli/tests/m126_requalification.rs`;
- `emissary-cli/tests/m127_token_lifetime.rs`;
- `emissary-cli/tests/m128_jsonrpc_batch.rs`;
- `emissary-cli/tests/m129_nonloopback_tls.rs`;
- `emissary-cli/tests/m130_post_corrective_requalification.rs`;
- a new `emissary-cli/tests/m138_post_lifecycle_requalification.rs` if a new durable current-head composition guard is the cleanest separation from historical M130 assertions.

Expected machine/planning/document paths:

- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml` only if exact current accepted path evidence needs truthful reconciliation;
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml` only for planning/test bookkeeping or exact current dependency/path evidence;
- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml` only if mechanical recomputation finds an actual authority inconsistency; expected count is unchanged;
- `plans/implementation/i2pcontrol-proposal-170/105-residual-option-audit.toml` only if its active residual accounting, rather than historical evidence, must be reconciled;
- `plans/implementation/i2pcontrol-proposal-170/110-completion-ledger.toml` for a post-M138 qualification marker if the ledger convention requires one;
- `plans/registry.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md` only to mark M130 historical/current-head supersession without rewriting its historical milestone narrative;
- `docs/i2pcontrol/README.md`;
- `docs/i2pcontrol/proposal-170-support.md`;
- `docs/i2pcontrol/tunnel-manager.md` only if current authority wording/counts are stale;
- `AGENTS.md` only if current support/authority text requires reconciliation;
- `plans/closure/i2pcontrol-proposal-170/138-closure.md` at closure.

Forbidden production changes for M138:

- `emissary-cli/src/**`;
- `emissary-core/src/**`;
- `emissary-util/src/**`;
- Cargo manifests/lockfile;
- Yosemite source/revision;
- `.github/**`;
- frontend/runtime configuration not itself a planning document.

A required change to any forbidden production path is a stop condition and requires a separately registered corrective implementation plan.

## 7. Work packages

### WP1 — Freeze current authority and create drift ledger

Before edits:

1. record exact HEAD and `git diff --stat`/cleanliness;
2. parse M095 and mechanically recompute its 840 TunnelManager cells;
3. prove declared and recomputed counts are `325/47/468`;
4. mechanically list the 47 blocked cells by option/family;
5. enumerate every active document that claims a current matrix or current runtime/security authority;
6. run the known historical/current-head guard suites individually and record every failure with classification:
   - genuine production/security regression;
   - current-head baseline drift;
   - historical milestone assertion incorrectly reused as current authority;
   - unrelated pre-existing toolchain/style drift.

No test may be edited before its failure is classified.

### WP2 — Separate historical milestone facts from current-head guards

Refactor/rebase stale tests so they preserve milestone evidence without freezing obsolete current state.

Preferred rules:

- milestone-specific commit ancestry checks remain historical;
- M127/M128/M129 behavior/security assertions remain exact;
- M130 may retain assertions about what M130 itself changed when scoped to the M130 implementation range;
- assertions about **current** matrix/support/docs must read the current M095 authority or assert `325/47/468` under M138;
- current containment must be governed by M061/M062 current manifests and exact accepted later seams, not M130's old `no core changes since 9948cfd...` rule;
- do not turn exact-path containment into prefix/glob approval;
- do not delete a failing guard without replacing its invariant where still applicable.

Add M138-specific durable current-head guards where this is clearer than overloading historical test files.

### WP3 — Integrated lifecycle composition requalification

Re-run and cross-check the entire lifecycle chain as one composition:

- M135 desired/base quantity behavior, no immediate purge, bounded latest-state control, LeaseSet desired-count truthfulness;
- M136 activity predicate at actual SAM/I2CP payload boundaries, reduction threshold/default/minimum, restore-on-activity, Streamr inclusion, shared-session aggregation;
- M137 close-before-reduce ordering, close<=reduce suppression, canonical teardown, first-wins neutral termination reason, manual/failure not mislabeled idle;
- M134 one-shot NewDest after proven idle close only, shared one-successor semantics, cancellation rollback, process/manual/restart/failure identity preservation.

At least one M138 integration test must exercise the complete conceptual sequence from idle reduction → idle close → authoritative `IdlePolicy` removal → successful resume/NewDest decision, using deterministic/fake runtime seams rather than wall-clock sleeps.

A separate negative integration must prove ordinary/manual stop/restart does not satisfy the NewDest gate.

### WP4 — Shared-control-plane/security requalification

Re-run M127–M130 invariants on current head and add M138 current-head assertions where needed:

- token lifetime and expiry outcome;
- bounded token/throttle state;
- bounded batch cardinality and sequential dispatch;
- per-element authentication; no intra-batch token sharing;
- notification/no-content behavior;
- loopback managed TLS only;
- complete explicit cert/key required for remote bind;
- rejection before bind/TLS-file/persistent-state side effects;
- TLS-only dispatch/no plaintext fallback;
- request/body/task/concurrency/time bounds;
- secret redaction.

Do not accept “unchanged since M130” as sufficient evidence; current-head tests must actually pass.

### WP5 — Containment/dependency current-head audit

Mechanically diff current head against the pinned M061 upstream baseline and reconcile with the exact M061 manifest.

For every non-I2PControl production path currently changed from upstream:

- it must already have an accepted neutral/composition owner and evidence;
- later M135/M137 composition/core seams must be individually named;
- no path may be allowed solely because M138 wants tests green;
- production source must remain free of Proposal/admin vocabulary outside the policy root;
- Yosemite exact optional alias and `subtle` ownership must remain isolated under M062;
- feature-disabled builds must not activate I2PControl-only dependencies/runtime behavior.

If M061/M062 reveal an unplanned production path, stop and classify it before any manifest expansion.

### WP6 — Active roadmap/document authority correction

Reconcile active authority wording so there is one unambiguous current state.

Required outcome after successful closure:

- registry names M138 as current runtime/security qualification authority;
- full-support roadmap states `325/47/468` everywhere it describes current state and describes 47, not 88, residual blockers;
- post-M114 roadmap is clearly historical for M127–M130 and says M130 was superseded for current-head qualification by M138, without rewriting M130 history;
- implementation README names M138 current authority and M130 historical qualification lineage;
- active user/developer docs retain **partial** Proposal support wording;
- no active document says M130 is the current-head authority after successful M138 closure;
- historical closure files may still say M130 was current at their own timestamp and are not rewritten.

### WP7 — Residual mechanical audit

Mechanically enumerate the remaining 47 blocked cells and ensure the active roadmap's residual-cluster prose matches the machine matrix.

Expected grouping at M138 start:

- `SigType`: 10;
- encrypted/authenticated LeaseSet cluster: 15;
- streaming `Profile`: 7;
- presentation `UseSSL`: 4;
- `UseOutproxyPlugin`: 4;
- HTTP `SSLProxies` + `JumpList`: 2;
- `UniqueLocalAddressPerClient`: 2;
- `MultiHoming` / `shouldBundleReplyInfo`: 2;
- Streamr `ConnectDelay`: 1;
- total: 47.

Do not hard-code this grouping into acceptance if M095's exact row structure demonstrates a different evidence-backed partition; M095 wins and the discrepancy must be explained. No residual is registered for implementation by M138.

### WP8 — Closure and successor decision

Write `138-closure.md` with:

- exact implementation/test/document commits;
- failure ledger before/after;
- requirement-to-evidence matrix;
- exact changed paths;
- current matrix hash/count evidence;
- containment/dependency diff evidence;
- integrated lifecycle and shared-control-plane results;
- compatibility/security review;
- unresolved findings with severity;
- explicit statement whether M138 supersedes M130 as current runtime/security authority.

If M138 closes cleanly, leave **no residual capability plan registered**. A separate planning pass may later select one of the 47 residual clusters.

## 8. Failure, cancellation, restart and contention review

Because M138 is qualification-only, it does not add lifecycle machinery. It must nevertheless re-prove the existing runtime behavior:

- reduction/close timers and reasons are generation-local and non-persistent;
- restart starts fresh idle state;
- stale generation controls/reasons cannot affect replacements;
- one shared SAM session has one activity/idle state and one NewDest successor policy;
- final-member release, manual stop/restart, network/SAM failure and idle close remain distinguishable;
- failed/cancelled NewDest resume does not consume eligibility or leak staged identity;
- control cells/trackers are bounded and have deterministic overload/eviction behavior;
- no lifecycle/store lock spans network or filesystem I/O;
- TLS rejection occurs before bind/filesystem side effects;
- concurrent JSON-RPC admission remains bounded.

Any failure here is a production defect, not documentation drift.

## 9. Compatibility and migration

Expected compatibility effect: none.

M138 must not alter:

- public JSON-RPC schema;
- Proposal names/types/actions;
- SAM/I2CP wire behavior;
- durable tunnel/control-state schema;
- secret-store format;
- Yosemite dependency revision;
- router configuration;
- external operational behavior.

Test/document changes may rebase current authority from M130 to M138. Historical closure and implementation commit references remain valid.

No migration is authorized or expected.

## 10. Focused regression requirements

At minimum add/retain deterministic evidence for:

1. M095 declared counts equal mechanical counts and equal `325/47/468`;
2. exactly 840 TunnelManager cells remain represented;
3. lifecycle promotions are exactly the M136 21 + M137 14 + M134 6 cells over the M131 post-refreeze baseline, with no unrelated promotion attributed to M138;
4. active docs all state current `325/47/468` or derive it from machine authority;
5. active docs retain partial-support wording;
6. M130 historical closure/commit lineage remains referenced but not called current-head authority after M138 closure;
7. M127 token expiry remains reachable and distinct from unknown token;
8. batch cap stays 32 and valid batches remain supported;
9. over-cap batch rejects before element side effects;
10. batch dispatch stays bounded/sequential and per-element authenticated;
11. managed TLS remains loopback-only;
12. rejected remote managed TLS has zero bind/cert/store side effects;
13. plaintext cannot reach dispatch;
14. current M061 changed-path set exactly equals accepted exact-path evidence;
15. no Proposal/I2PControl business vocabulary leaks into neutral core/composition paths;
16. M062 exact Yosemite/subtle dependency ownership remains unchanged;
17. feature-disabled CLI check remains clean;
18. M135 desired quantity decrease/restore and LeaseSet desired-count coherence remain green;
19. M136 reduce/restore activity contract remains green for streaming and datagram/Streamr;
20. M137 close ordering/reason semantics remain green;
21. M134 dedicated proven-idle resume rotates once;
22. M134 shared proven-idle resume creates one successor;
23. manual stop/start/restart/failure does not rotate NewDest;
24. failed/cancelled resume preserves last committed identity and retryability;
25. integrated reduction→close→IdlePolicy→resume/NewDest sequence succeeds deterministically;
26. integrated manual stop/restart negative sequence never becomes idle eligibility;
27. remaining blocked-cell mechanical total is 47;
28. no blocked residual was silently implemented by test/doc edits.

## 11. Broad verification

Run from a clean worktree after all test/document corrections.

### Core / workspace

```text
cargo check -p emissary-core
cargo test -p emissary-core --no-fail-fast
cargo clippy -p emissary-core --all-targets -- -D warnings
cargo check
```

### CLI feature isolation

```text
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
```

### Full I2PControl library/integration

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test persistence_concurrency --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test adversarial --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test router_info_truthfulness --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test client_services_integration --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test client_services_live --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
```

### Historical + current qualification/containment guards

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol \
  --test m060_containment \
  --test m061_containment \
  --test m062_dependency_containment \
  --test m095_full_support_matrix \
  --test m105_residual_option_audit \
  --test m126_requalification \
  --test m127_token_lifetime \
  --test m128_jsonrpc_batch \
  --test m129_nonloopback_tls \
  --test m130_post_corrective_requalification \
  --test m138_post_lifecycle_requalification \
  --no-fail-fast
```

If no dedicated M138 test file is needed after refactoring, omit only that one target and explain why the same durable current-head guard is provided elsewhere.

### Static/tooling

```text
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

`cargo fmt --all -- --check` is evidence-only where the repository's known stable/nightly rustfmt mismatch remains. Do not run a bulk formatter and retain unrelated churn. If clippy still fails only on the previously recorded untouched `chunks_exact_to_as_chunks` lint, prove changed M138 Rust test files are clean and record the exact pre-existing failure rather than weakening lint policy.

## 12. Documentation/static-guard requirements

Closure must prove:

- registry, implementation README and full-support roadmap agree on current matrix and authority;
- post-M114 roadmap is labeled historical for current-head authority after M138;
- current support docs say partial, not full;
- M095/M105 tests agree with machine artifacts;
- no active document retains `284/96/460` or `284/88/468` as the **current** matrix;
- historical closure files are not rewritten solely to remove their historical counts;
- M061/M062 exact-path/dependency guards remain at least as strict as before;
- no new broad `emissary-core/**`/`emissary-cli/src/**` allowlist is introduced;
- no new dependency or feature activation appears.

## 13. Security review required at closure

Explicitly review:

- authentication token lifetime/capacity/redaction;
- JSON-RPC batch cardinality/authentication/notification/fanout;
- TLS bind identity/fail-closed behavior;
- request/task/body/deadline bounds;
- I2PControl runtime enable/disable composition;
- neutral core policy vocabulary containment;
- pool target isolation and LeaseSet truthfulness;
- idle activity/close reason spoof-resistance;
- NewDest eligibility spoof/replay/race resistance;
- staged secret rollback and log/debug redaction;
- shared-session identity/policy equality;
- restart/manual/failure semantics;
- dependency isolation and exact Yosemite pin.

No “inherited from M130/M134” row may be marked pass without current-head executable or static evidence.

## 14. Acceptance criteria

M138 closes as complete only when all are true:

1. current head is explicitly recorded;
2. M095 mechanically recomputes to exactly `325/47/468` with 840 cells;
3. no Proposal cell is promoted by M138;
4. current-head qualification/containment guards are green after evidence-backed baseline corrections;
5. M127–M129 security behavior is green on current head;
6. M135–M137 and M134 lifecycle behavior is green individually and in integrated composition;
7. live/adversarial/production/persistence/client-services/router-info suites are green or any skipped suite has an explicit environment-only reason and equivalent evidence;
8. M061/M062 current exact-path/dependency containment is green;
9. feature-disabled build is green;
10. active docs/registry/roadmaps all agree on `325/47/468`, partial support, and M138 current authority;
11. post-M114 roadmap clearly treats M130 as historical current-head authority superseded by M138;
12. no forbidden production source/dependency path changed under M138;
13. no unresolved high/medium correctness or security finding remains;
14. closure contains exact commands/results and before/after drift ledger;
15. closure explicitly supersedes M130 for current-head runtime/security qualification while retaining M130 historical evidence;
16. no residual capability successor is automatically registered.

## 15. Stop conditions

Stop and close as blocked/corrective-required if any of the following occurs:

- a current `apply` cell lacks real runtime behavior;
- a high/medium security or correctness defect is found;
- lifecycle integration needs production Rust changes;
- current M061/M062 containment can only be made green by broadening a path prefix/glob without prior accepted owner evidence;
- dependency isolation or exact Yosemite pin has drifted;
- a historical test failure cannot be classified without changing production semantics;
- active matrix/docs cannot be reconciled without changing support claims;
- full suite reveals a regression that requires runtime code changes;
- external/upstream mutation would be required.

When stopped for a production defect, write a narrowly scoped corrective plan naming the defect and exact owner. Do not fix it inside M138.

## 16. Closure evidence required

`plans/closure/i2pcontrol-proposal-170/138-closure.md` must include:

- baseline/current implementation commits;
- exact files changed by M138;
- before-edit failing-test/contradiction ledger;
- classification of every stale historical guard changed;
- requirement-to-evidence matrix;
- M095 recomputation and hash/count evidence;
- exact remaining 47-cell residual inventory;
- M061 upstream diff vs accepted exact-path manifest;
- M062 dependency/pin evidence;
- current-head M127–M129 security test results;
- M135/M136/M137/M134 focused + integrated lifecycle results;
- production/live/adversarial/persistence/client-services/router-info results;
- feature-disabled and workspace checks;
- clippy/rustfmt/diff-check outcomes with pre-existing-only evidence separated;
- failure/cancellation/restart/contention review;
- compatibility/migration review;
- security review;
- documentation/authority reconciliation evidence;
- unresolved findings with severity;
- explicit disposition of M130 current-head authority;
- internal-only/read-only-upstream attestation.

## 17. Successor policy

A successful M138 closure establishes a clean `325/47/468` current-head baseline and leaves Proposal 170 partial.

M138 must **not** choose or register the next residual capability cluster during implementation or closure. Selection of the next cluster is a separate planning decision after the requalification evidence is available.

Historical M132/M133 blocked attempts and M130 qualification remain preserved as historical records.
