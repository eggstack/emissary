# M105 — Residual TunnelManager Option Primitive and Applicability Audit

Status: **closed**

Class: invariant / infrastructure audit; no production behavior

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Corrective predecessors:

- M097 common session/key option closure — `plans/closure/i2pcontrol-proposal-170/097-closure.md`
- M098 client/proxy/HTTP option closure — `plans/closure/i2pcontrol-proposal-170/098-closure.md`
- M099 server/access/throttle closure — `plans/closure/i2pcontrol-proposal-170/099-closure.md`
- M104 full-support reclosure — `plans/closure/i2pcontrol-proposal-170/104-closure.md`

Repository baseline:

- `aa90c3afc830dcdbca8f6bf8acb5737acc73c366` — M104 closed as blocked.

Pinned external contract:

- I2P Proposal 170, `I2PControl Expansion`;
- pinned revision: `2026-05-20`;
- proposal status at the baseline: `Open`.

Canonical/internal authority:

- `plans/000-long-term-specification.md`;
- `plans/001-terminology-and-domain-model.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- ADR-0001/0002/0003/0004;
- M061/M062/M063 containment authority;
- M093 tunnel production/security reclosure;
- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`;
- M104 closure's final matrix hash and residual accounting.

## 1. Objective

Audit every applicable TunnelManager cell that M104 left as `blocked_primitive` and determine, with exact repository and reference evidence, whether the blocker is:

1. implementable entirely inside the existing `emissary-cli/src/i2pcontrol/**` ownership boundary;
2. implementable only through a small neutral existing-owner seam outside I2PControl that can be named and bounded before coding;
3. blocked by a concrete Yosemite/SAM/dependency capability that does not exist in the accepted dependency surface;
4. blocked by a genuine missing Emissary architecture/owner that would require a separately approved architecture decision;
5. incorrectly classified as applicable and therefore a candidate for an evidence-backed `not_applicable` correction; or
6. semantically ambiguous enough that no implementation plan is safe until the pinned Proposal 170/reference behavior is resolved.

M105 is an audit and planning milestone. It MUST NOT implement any blocked option, change runtime behavior, add a dependency, patch/fork/vendor Yosemite, or widen core ownership.

The purpose is to replace the current coarse residual blocker ledger with an exact, machine-readable decision surface from which later implementation plans can be safely derived.

## 2. Why this corrective audit is required

M097-M099 correctly stopped at missing primitives and M104 correctly refused a full-support claim, but those passes had different primary responsibilities:

- M097 tested the common session/key path and recorded immediate primitive failures;
- M098 implemented the client/proxy/HTTP subset that was already clearly owned by existing I2PControl backends;
- M099 implemented the server access/filter/throttle subset already owned by accepted server runtime paths;
- M104 verified the integrated state and counted the remaining blockers.

They did not perform one exhaustive cross-reference audit asking, for every residual cell, whether the current blocker is still the narrowest truthful classification when compared against:

- the exact pinned Proposal 170 semantics;
- Java I2P/I2PTunnel reference behavior;
- i2pd/I2PControl behavior where relevant;
- the current Yosemite API and actual SAM wire serialization;
- existing Emissary I2PControl runtime ownership;
- existing neutral lower-layer owners;
- accepted security/anonymity constraints.

M104 therefore closed with a correct blocker count but without enough evidence to decide whether any residual family can now be safely decomposed into a contained successor implementation plan.

M105 supplies that missing decision evidence. It does not weaken M104's closure or full-support gate.

## 3. Baseline residual inventory

M104 records the authoritative baseline:

- 70 canonical TunnelManager option rows;
- 12 canonical tunnel types;
- 840 total cells;
- 218 `apply`;
- 164 applicable `blocked_primitive`;
- 458 `not_applicable`;
- 0 `planned_apply`;
- 0 unknown/unsupported/accept-inert.

The 164 blocked cells are currently grouped as:

| Residual family | Blocked cells | Current blocker authority |
|---|---:|---|
| `Shared` | 7 | M097 shared-session ownership/handoff |
| `UseSSL` | 4 | Yosemite/SAM session wire |
| `TunnelVariance`, `TunnelBackupQuantity`, `SigType`, `CustomOptions` | 40 | Yosemite/SAM session wire |
| `NewDest`, `PersistentClientKey` | 14 | destination/key lifecycle |
| `PrivKeyFile` | 10 | confined key import/store/handoff |
| `UseOutproxyPlugin`, `SSLProxies`, `JumpList` | 12 | no bounded accepted proxy owner |
| `ConnectDelay`, `Profile`, `DelayOpen`, `Reduce*`, `Close*` | 56 | no exact client session lifecycle owner |
| `AllowInternalSSL`, `UniqueLocalAddressPerClient`, `MultiHoming` | 6 | no accepted TLS/address-routing owner |
| `EncryptLeaseSet`, `OptionalLookup`, `LeaseSetClientAuths` | 15 | no supported LeaseSet serializer/key handoff |

M105 MUST reconcile exactly 164 unique `(canonical option, canonical tunnel type)` cells against the M104-reviewed M095 matrix hash. Missing, duplicated, newly invented, or silently dropped cells are audit failures.

## 4. Deliverable

Create:

`plans/implementation/i2pcontrol-proposal-170/105-residual-option-audit.toml`

The file is a planning/evidence artifact, not a runtime configuration file.

It MUST contain one record per M104 `blocked_primitive` cell and a summary section that reproduces the exact 164-cell input inventory.

### 4.1 Required per-cell fields

Each cell record MUST include at least:

- canonical option key;
- canonical tunnel type;
- Proposal 170 value type and pinned semantic summary;
- current M095 blocker text / blocking milestone;
- exact current Emissary backend/runtime owner, if any;
- exact current Yosemite/SAM primitive or wire path relevant to the cell, if any;
- reference implementation behavior and evidence source;
- whether the option is truly applicable to this tunnel family under the pinned contract;
- required runtime effect for a truthful `apply` classification;
- candidate implementation owner;
- exact candidate production paths if a contained implementation appears possible;
- whether any non-I2PControl production path would be required;
- whether any dependency/Cargo.lock change would be required;
- whether an ADR/architecture decision would be required;
- security/anonymity implications;
- persistence/key/secret/path implications where relevant;
- recommended audit disposition;
- proposed successor grouping, if any;
- evidence references sufficient for closure review.

No private key material, credentials, tokens, destination private data, or file contents may be embedded in the audit artifact.

### 4.2 Audit disposition vocabulary

Use only these final audit dispositions:

- `i2pcontrol_local_candidate` — exact semantics appear implementable using existing I2PControl-owned runtime/data structures, with no dependency or core change;
- `neutral_owner_candidate` — exact semantics appear implementable only with a minimal Proposal-170-agnostic change to an existing canonical lower-layer owner; exact path/owner must be named, but this disposition does not authorize the change;
- `dependency_blocked` — required behavior is absent from the current accepted Yosemite/SAM/dependency surface and cannot truthfully be recreated at the I2PControl layer;
- `architecture_decision_required` — support would require a new owner/subsystem or a material change to accepted lifecycle/security architecture;
- `not_applicable_candidate` — pinned/reference semantics demonstrate that the current applicability classification is probably incorrect for this tunnel family; requires explicit matrix correction evidence before changing support counts;
- `semantic_blocked` — Proposal/reference semantics remain ambiguous or contradictory enough that implementation would be speculative.

Do not use `supported`, `apply`, or equivalent completion language in M105. M105 only audits readiness and applicability.

## 5. Evidence hierarchy

For each residual, evaluate evidence in this order:

1. pinned Proposal 170 revision `2026-05-20`;
2. current Emissary production behavior and accepted M093/M097-M104 closures;
3. Java I2P/I2PTunnel behavior for the corresponding option;
4. current Yosemite source and actual SAM wire construction;
5. i2pd/I2PControl or I2P+ behavior where it clarifies the Proposal 170 contract;
6. other read-only external implementation/discussion evidence only when needed to resolve ambiguity.

Reference implementations inform semantics but do not automatically dictate Emissary architecture. Java-specific plugin/UI/storage concepts must not be copied merely because they exist in Java I2P.

If external implementations disagree, record the disagreement and prefer the pinned Proposal 170 text. If the proposal itself does not resolve it, use `semantic_blocked`; do not guess.

All external repositories and maintainer channels are read-only evidence.

## 6. Required audit work packages

### WP1 — Freeze the exact input inventory

Before researching implementation paths:

1. read the M104 closure and M095 matrix;
2. verify the recorded M104 SHA-256 for the matrix or record an exact reason if repository metadata has changed without cell-semantic change;
3. enumerate all 164 `blocked_primitive` cells;
4. verify no `planned_apply`, unsupported, unknown, or accept-inert cell exists;
5. record the current production head and planning head separately;
6. do not mutate the M095 support dispositions during this work package.

If the input count is not 164, stop and reconcile the discrepancy before continuing.

### WP2 — Audit Proposal 170 applicability and semantics

For every residual option:

- determine which of the 12 tunnel types the pinned proposal actually makes the option meaningful for;
- distinguish an explicit Proposal 170 option from a Java I2PTunnel implementation detail;
- identify conditional semantics that make a cell meaningful only when another option or data plane exists;
- check whether M095's current applicability assignment is too broad or too narrow;
- document exact expected observable/runtime behavior.

A `not_applicable_candidate` requires affirmative evidence. Mere implementation difficulty is not evidence of non-applicability.

### WP3 — Audit existing I2PControl-local ownership

Inspect the current `emissary-cli/src/i2pcontrol/**` runtime for each residual and answer:

- is there already a lifecycle owner that can express the exact behavior?
- is the missing behavior merely validation/plumbing, or would it require a new independent runtime subsystem?
- can state be bounded per tunnel generation/name?
- can restart/edit/stop semantics be exact and cancellation-safe?
- can secrets/paths be confined and redacted?
- would the implementation preserve direct-I2P/no-clearnet-DNS and literal-loopback server boundaries?

`i2pcontrol_local_candidate` is allowed only when exact candidate paths and lifecycle behavior can be named now.

### WP4 — Audit Yosemite/SAM capability rather than Rust surface names

For session/key/LeaseSet residuals, inspect the current accepted Yosemite revision and actual SAM command serialization.

Do not infer support merely because a Rust `SessionOptions` field exists.

For each relevant option, record:

- public/internal Yosemite field or API;
- whether it is consumed;
- exact SAM `SESSION CREATE`/destination/key command emitted;
- whether the value reaches the router wire path;
- whether unsupported values are hardcoded, omitted, or transformed;
- whether a supported existing API can already express the required Proposal 170 behavior without dependency modification.

If the required wire behavior is absent, classify `dependency_blocked` unless a fully equivalent I2PControl-local path already exists.

M105 MUST NOT patch Yosemite, add `[patch]`, vendor source, change dependency versions, or prepare an upstream contribution.

### WP5 — Audit shared-session and destination/key lifecycle ownership

For `Shared`, `NewDest`, `PersistentClientKey`, `PrivKeyFile`, `Profile`, and any dependent client-management option:

- identify the exact current owner of destination/session creation and destruction;
- determine whether sharing/persistence can be scoped to the existing I2PControl control plane without cross-subscriber single-owner receiver violations;
- determine whether a bounded key store/import facility already exists;
- determine whether restart/edit semantics can preserve identity exactly;
- identify filesystem confinement, permissions, atomicity, secret-zeroization/redaction, and symlink/special-file requirements;
- distinguish ephemeral destination generation from persistent private-key lifecycle.

If implementing the option would require inventing a router-wide destination/key service or broad persistent key subsystem, classify `architecture_decision_required`, not `i2pcontrol_local_candidate`.

### WP6 — Audit proxy/plugin/TLS/jump-list semantics

For `UseOutproxyPlugin`, `SSLProxies`, and `JumpList`:

- determine the precise Proposal/reference meaning;
- determine whether Java-specific plugin architecture is contract-essential or an implementation mechanism;
- inspect whether existing I2PControl HTTP/CONNECT/SOCKS backends can express an equivalent bounded behavior without direct clearnet fallback;
- define trust validation for any TLS outproxy path;
- define deterministic bounded selection/failover requirements for any jump-list semantics;
- preserve local/private/unspecified destination rejection and explicit-I2P-outproxy requirements.

Do not add a plugin framework solely to satisfy a boolean option.

### WP7 — Audit client-management lifecycle semantics

For `ConnectDelay`, `DelayOpen`, `Reduce`, `ReduceCount`, `ReduceTime`, `Close`, `CloseTime`, and `Profile`:

- compare the pinned/reference semantics with the actual Emissary tunnel-generation lifecycle;
- distinguish router tunnel-pool controls from application listener/session timers;
- reject approximations where an application timeout would merely look similar to a router/session control;
- determine whether an exact generation-local I2PControl implementation exists;
- identify interactions with edit/restart, cancellation, and shared session identity.

If exact semantics require a lower-level pool/session owner absent from I2PControl, classify the correct blocker rather than implementing an approximate timer.

### WP8 — Audit server presentation/address-routing semantics

For `AllowInternalSSL`, `UniqueLocalAddressPerClient`, and `MultiHoming`:

- determine exact Proposal/reference semantics;
- test applicability against Emissary's accepted literal-loopback local-target model;
- identify whether safe support would require TLS termination/trust ownership, local-address allocation, or request-selected routing;
- verify whether such behavior would reopen M093 anonymity/resource boundaries;
- record whether an equivalent safe I2PControl-local mechanism already exists.

No audit conclusion may weaken loopback confinement or authorize LAN/request-selected target routing simply for option parity.

### WP9 — Audit LeaseSet semantics and confidentiality

For `EncryptLeaseSet`, `OptionalLookup`, and `LeaseSetClientAuths`:

- identify the exact LeaseSet/SAM option semantics and supported modes;
- inspect the accepted Yosemite/SAM serialization and key handoff path;
- determine whether required keys/auth lists have an existing bounded owner;
- distinguish metadata acceptance from actual encrypted/authenticated LeaseSet publication;
- verify that no unsupported mode can silently fall back to a public LeaseSet;
- identify whether support requires an existing neutral core owner, a dependency capability, or new architecture.

Any silent confidentiality/authentication downgrade is a hard audit failure.

### WP10 — Group only evidence-ready successor slices

After all 164 cells are classified, group candidate future work by coherent owner rather than by arbitrary option count.

A successor grouping may be recommended only when:

- every included cell has identical or compatible ownership/lifecycle assumptions;
- exact production path budget can be named;
- security invariants and failure semantics are stated;
- no unresolved semantic question exists;
- dependency/core expansion is not being hidden inside the group.

Examples of acceptable output are:

- one contained I2PControl-local option slice;
- one narrowly budgeted neutral-owner proposal requiring a maintainer architecture decision;
- one dependency-blocked ledger that remains intentionally unplanned.

M105 MUST NOT implement these successors. It SHOULD NOT pre-register multiple future implementation plans. Closure may advance only the single next dependency-ready handoff, if one actually exists.

## 7. Production and path budget

M105 authorizes **no production source changes**.

Expected changed paths are limited to planning/evidence/test guard surfaces:

- `plans/implementation/i2pcontrol-proposal-170/105-residual-tunnel-option-primitive-audit.md`;
- `plans/implementation/i2pcontrol-proposal-170/105-residual-option-audit.toml`;
- `plans/closure/i2pcontrol-proposal-170/105-closure.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`;
- `plans/registry.md`;
- `emissary-cli/tests/m062_dependency_containment.rs` only as required to register the exact new planning/evidence paths;
- optionally `emissary-cli/tests/m105_residual_option_audit.rs` if a lightweight machine-readable coverage guard is needed.

No file under `emissary-cli/src/**`, `emissary-core/**`, `emissary-util/**`, Cargo manifests, `Cargo.lock`, frontend code, or workflow configuration may change under M105.

If the audit itself appears to require a production change to determine semantics, stop and record that as missing evidence; do not perform the change.

## 8. Failure, cancellation, restart, and contention semantics

M105 changes no runtime, so runtime cancellation/contention behavior must remain byte-for-byte unaffected.

Audit-process requirements:

- the TOML artifact must be deterministic from the checked repository/spec evidence;
- partial audit output must not be represented as closure;
- if interrupted, work resumes from the last reviewed cell without changing support classifications;
- conflicting evidence must be recorded, not resolved by majority vote;
- concurrent repository changes affecting M095, Yosemite dependency revision, or tunnel runtime ownership require a baseline recheck before closure;
- if the pinned Proposal 170 revision changes during the audit, stop and perform a separate revision delta decision before mixing revisions.

## 9. Compatibility and migration

M105 has no runtime compatibility or storage migration effect.

It MUST NOT:

- change accepted JSON-RPC request/response behavior;
- change supported/blocked option responses;
- change tunnel persistence;
- alter destination/key material;
- alter default or feature-disabled builds;
- change dependency resolution.

Any future implementation recommendation must state its own compatibility/migration effects in a new plan.

## 10. Security and anonymity review requirements

Every residual classification must explicitly consider where relevant:

- peer identity trust source;
- direct-I2P versus clearnet/outproxy routing;
- DNS leakage;
- local/private target exposure;
- TLS trust/termination;
- private key storage/import and secret disclosure;
- LeaseSet confidentiality/client authorization;
- shared-session cross-tunnel identity coupling;
- resource cardinality and unbounded maps/queues;
- timing/lifetime changes that could make server tunnels easier to fingerprint or exhaust;
- restart/edit generation isolation;
- path traversal/symlink/special-file behavior.

A candidate that achieves literal option behavior by weakening an accepted M093 anonymity or resource invariant is not implementation-ready. Classify it `architecture_decision_required` or `dependency_blocked` as appropriate.

## 11. Focused verification

At minimum run:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check
git diff --check
```

If `m105_residual_option_audit.rs` is created, also run it explicitly.

The audit guard, whether implemented as a dedicated test or a minimal extension of an existing matrix test, should verify only high-value invariants:

- exactly 164 input residual cells from the M104 baseline;
- each cell appears exactly once;
- every disposition is from the six-value audit vocabulary;
- every `i2pcontrol_local_candidate` or `neutral_owner_candidate` names exact candidate paths/owners;
- every `dependency_blocked` names the missing dependency/wire capability;
- every `not_applicable_candidate` has affirmative Proposal/reference evidence;
- no audit record contains an `apply`/supported completion claim.

Do not add broad CI, fuzzing, soak, coverage, or generated verification infrastructure for this planning artifact.

The known repository rustfmt stable/nightly mismatch is not M105 scope. Do not reformat audited unrelated core files merely to make `fmt` green.

## 12. Documentation and static guards

Update the active registry, full-support roadmap, and implementation handoff index to make M105 the sole ready handoff.

The registry must state that:

- M104 remains closed as blocked;
- 164 residual cells remain blocked in production throughout M105;
- M105 is evidence-only and does not itself reduce the blocker count;
- no successor implementation plan is ready before M105 closure;
- only one next successor may be registered after closure, and only if its path is genuinely bounded.

Update the exact M062 planning-path allowlist for M105 artifacts without widening production globs or dependency permissions.

Do not change end-user Proposal 170 support documentation to imply greater support during M105.

## 13. Acceptance criteria

M105 may close only when all are true:

1. all 164 M104 residual cells are present exactly once in `105-residual-option-audit.toml`;
2. every cell has an applicability decision, exact semantics, current owner/blocker, security review, and one allowed audit disposition;
3. every `i2pcontrol_local_candidate` names exact existing I2PControl paths and demonstrates how the required behavior would be real rather than inert/approximate;
4. every `neutral_owner_candidate` names the exact canonical owner and minimal neutral path budget, and is explicitly marked as requiring separate approval before implementation;
5. every `dependency_blocked` identifies the exact missing Yosemite/SAM/dependency behavior rather than merely citing a version number;
6. every `architecture_decision_required` states why existing owners cannot safely express the feature;
7. every `not_applicable_candidate` has affirmative pinned/reference evidence and is not justified by implementation difficulty;
8. every `semantic_blocked` records the unresolved conflict/ambiguity;
9. M095 production support dispositions remain unchanged during the audit;
10. no production source, dependency, Cargo.lock, frontend, workflow, or upstream repository changed;
11. containment guards pass for the planning/evidence changes;
12. the closure record identifies whether a bounded successor implementation plan exists and, if so, recommends exactly one next handoff;
13. Proposal 170 support remains described as partial unless and until a future M104 reattempt succeeds.

## 14. Stop conditions

Stop rather than broaden scope if:

- the M104 residual input cannot be reconciled to exactly 164 cells;
- the Proposal 170 pinned revision changed and the delta affects option semantics;
- determining support would require modifying Yosemite or core first;
- a candidate requires a new dependency merely to explore feasibility;
- a proposed local implementation would weaken M093 anonymity/security boundaries;
- reference implementations conflict materially and the proposal does not resolve the behavior;
- a successor grouping cannot name an exact owner/path budget;
- the only path to full support is a material architecture change not covered by ADR-0004.

A stop condition is a valid audit result. It must be recorded in the closure rather than bypassed.

## 15. Closure evidence required

Create:

`plans/closure/i2pcontrol-proposal-170/105-closure.md`

The closure MUST include:

- exact repository head and M104/M095 input evidence;
- hash of `105-residual-option-audit.toml`;
- 164-cell coverage accounting;
- disposition counts and residual-family summary;
- requirement-to-evidence matrix;
- external read-only sources consulted;
- exact verification commands/outcomes;
- containment and dependency review;
- security/anonymity review;
- unresolved semantic/dependency/architecture blockers;
- successor grouping recommendations;
- explicit decision whether any one successor is dependency-ready;
- internal-only attestation;
- final disposition: `closed`, `corrective pass required`, or `blocked`.

If no contained successor exists, M105 may close with the residual line still blocked. That is preferable to manufacturing an implementation path.

If one or more candidates exist, closure should register only the single highest-value/coherent next implementation handoff. Other candidate groups remain roadmap/deferred material until dependencies close.

## 16. Internal-only boundary

All writes remain inside `eggstack/emissary`.

External Proposal 170, Java I2P, i2pd, I2P+, Yosemite, issue, commit, pull-request, and source material is read-only evidence.

M105 MUST NOT:

- open, update, comment on, or prepare an upstream issue/PR/review;
- request maintainer feedback or adoption;
- push any branch/tag/patch/release outside this fork;
- prepare an upstream contribution package or merge plan;
- treat public licensing or an external source reference as submission authority.

The audit is solely for internal architectural and implementation planning.
