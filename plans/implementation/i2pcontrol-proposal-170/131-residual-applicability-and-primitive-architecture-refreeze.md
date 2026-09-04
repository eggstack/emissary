# M131 — Residual Applicability and Primitive-Architecture Re-freeze

Status: **closed as blocked**

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Current qualification authority:

- `plans/closure/i2pcontrol-proposal-170/130-closure.md`
- M130 implementation head `fe1a981`
- M130 closure head / M131 production-behavior baseline `a68094e128d2b92f0fd5b350e38512ef6b65cb6b`

Pinned Proposal authority:

- I2P Proposal 170, `I2PControl Expansion`
- revision `2026-05-20`
- status `Open`

Planning-time reference snapshots to re-check during execution:

- Java I2PControl Proposal-170 implementation PR `i2p/i2p.plugins.i2pcontrol#6`, inspected at head `45bb593000408071dd376b78848fdc246dccd964`;
- Java I2P/I2PTunnel reference source, planning-time snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`;
- internal Yosemite I2PControl fork Y005, exact accepted revision `59140a2277bf296928d2e8ce39a148182eeff044`, consumed only through the optional exact `yosemite-i2pcontrol` alias.

All external/upstream sources are read-only evidence. This plan authorizes writes only to `eggstack/emissary`. It does not authorize an upstream issue, PR, review, comment, contact, merge, release, adoption request, or contribution workflow.

## 1. Objective

Re-freeze the exact semantics, applicability, lower-layer ownership and dependency readiness of all 96 currently blocked TunnelManager cells before any further capability implementation.

M131 exists because the current `blocked_primitive` inventory is an evidence ledger, not an architecture contract. Subsequent direct comparison with the pinned Proposal, the Java Proposal-170 implementation work, current I2PTunnel runtime/configuration semantics, Yosemite Y005 and Emissary's actual lower-layer owners indicates that several residual blocker descriptions may be too broad or attached to the wrong owner. Examples found during planning include:

- `MultiHoming` appears to map to `shouldBundleReplyInfo`, not a host-interface multihoming subsystem;
- `Profile=interactive` appears to map to bounded streaming-window behavior, not router-global peer-selection policy;
- `SSLProxies` and `JumpList` appear to be HTTP-client-specific reference behavior rather than generic proxy-family behavior;
- `UseSSL` applicability and presentation role need a fresh type-by-type freeze before any TLS implementation;
- Streamr `Profile`, `DelayOpen`, `ConnectDelay`, `Close*`, `NewDest` and reduction cells require exact applicability review rather than inheritance from TCP-client assumptions;
- Yosemite Y005 now provides typed encrypted-LeaseSet/client-auth SAM serialization, so the remaining LeaseSet blockers must be re-described at the actual Emissary crypto/NetDB/session owner rather than stale serializer language.

The goal is not to reduce the blocked count. The goal is to make the residual ledger exact enough that the next implementation plans can be narrow, dependency-ready and security-reviewable.

## 2. Required outcome

M131 MUST produce all of the following:

1. a cell-complete evidence table covering every one of the 96 blocked cells at the M130 baseline;
2. an exact per-cell disposition of:
   - retain `blocked_primitive` with corrected primitive/owner;
   - reclassify to `not_applicable` only with affirmative pinned/reference evidence;
   - `apply` is forbidden in M131 because no runtime capability implementation is authorized;
3. a mechanically recomputed M095 matrix and reconciled M105/M110 ledgers if any applicability correction is justified;
4. a neutral lower-layer primitive map showing which cells share a real missing capability and which should remain I2PControl-local;
5. an ordered dependency graph for future M132+ implementation work, including exact candidate production path budgets and stop conditions;
6. one next dependency-ready implementation handoff recommendation at closure, or an explicit statement that none is dependency-ready.

M131 MUST NOT implement any of the missing capabilities itself.

## 3. Baseline and scope freeze

### 3.1 Production baseline

M130 is the current-head qualification authority and mechanically records:

- `284 apply`;
- `96 blocked_primitive`;
- `460 not_applicable`.

M127-M130 changed no Proposal option cell. M131 starts from that support state.

The plan file is created after the M130 closure commit, so the execution agent MUST distinguish:

- **production-behavior baseline**: `a68094e128d2b92f0fd5b350e38512ef6b65cb6b`;
- **execution head**: the actual merged `master` containing this registered M131 plan and its registry/roadmap/index updates.

If production code has changed after the production-behavior baseline before M131 execution, stop and re-freeze the baseline before drawing cell conclusions.

### 3.2 Current residual partition to exhaustively account for

The starting 96 are currently recorded as:

- 4 `UseSSL` cells;
- 10 `SigType` cells;
- 63 client proxy/profile/reduction/lifecycle cells, including 18 TCP `Close`/`CloseTime`/`NewDest` cells plus Streamr residuals;
- 19 server presentation/routing/LeaseSet cells.

M131 MUST derive the exact 96-cell list mechanically from `095-full-support-matrix.toml`; this prose partition is only a cross-check.

## 4. Invariants

M131 MUST preserve all canonical Proposal-170 invariants, especially:

- parser acceptance, raw persistence, serializer reachability or dormant struct fields are not runtime capability evidence;
- a cell is `apply` only when the requested setting has the exact externally observable runtime effect;
- a cell becomes `not_applicable` only from affirmative Proposal/reference applicability evidence, never because implementation is difficult;
- unsupported supplied values remain fail-before-effect and cannot silently coerce/default;
- no direct-clearnet fallback for I2P destinations;
- clearnet proxying remains explicit and I2P-routed through the accepted outproxy boundary;
- server local-target confinement remains literal-loopback unless a separately accepted neutral primitive proves an equally strong confinement model;
- no secret/key/path may become visible in RPC output, logs, `Debug`, RouterInfo, RawConfig or planning fixtures;
- shared-session compatibility, cancellation, generation isolation and restart atomicity from M110/M116/M123 remain authoritative;
- Yosemite remains the sole accepted SAM implementation for the I2PControl path;
- Proposal-specific business/admin/application policy stays under `emissary-cli/src/i2pcontrol/**` wherever possible;
- any future production change outside I2PControl must be neutral, canonically owned, minimal, path-budgeted and separately authorized;
- no unrelated base I2PControl parity is in scope.

## 5. Explicit non-goals

M131 MUST NOT:

- modify runtime/production Rust code;
- modify `eggstack/yosemite`;
- add, remove or change Cargo dependencies/features;
- add a new plugin system, TLS stack, crypto algorithm, LeaseSet implementation, NetDB behavior, tunnel-pool policy, SAM protocol extension or generic router subsystem;
- promote any blocked cell to `apply`;
- implement `GetKeys`, `GetRate`, `RouterManager`, `NetworkSetting`, `AdvancedSettings` or other unrelated base methods;
- weaken M061/M062 containment or M093 security boundaries;
- use matrix-count reduction as an acceptance criterion;
- contact or mutate upstream/reference repositories.

Permitted repository changes are planning/evidence/documentation only, as detailed in §10.

## 6. Work packages

### WP1 — Mechanically freeze the 96 starting cells

Generate an exact machine-readable or tabular inventory from M095 containing for every blocked cell:

- canonical option key;
- canonical tunnel family;
- Proposal type/semantics;
- current blocker text;
- current completion owner/milestone;
- security/secret/path/identity flags;
- current production validation behavior;
- M130/M121/M125 historical evidence that caused the present disposition.

Acceptance:

- exactly 96 unique starting cells;
- no blocked cell omitted, duplicated or grouped without preserving cell identity;
- counts reconcile exactly with M095 before semantic review begins.

### WP2 — Re-freeze client proxy/outproxy applicability

Review at minimum:

- `UseOutproxyPlugin`;
- `SSLProxies`;
- `JumpList`;
- any neighboring proxy fields whose applicability evidence is needed to interpret those three.

For each of `httpclient`, `socks`, `socksirc`, and `connectclient`, determine whether the reference data plane actually consumes the option or whether prior matrix applicability was inherited too broadly.

Required evidence hierarchy:

1. pinned Proposal text/table;
2. Proposal-170 Java request parser/creator at a recorded exact head;
3. actual I2PTunnel runtime/configuration owner at a recorded exact source snapshot;
4. current Emissary data-plane ownership.

Special questions:

- Is `SSLProxies` specifically the HTTP CONNECT/SSL-outproxy set rather than a generic SOCKS/CONNECT field?
- Is `JumpList` specifically HTTP address-helper/jump-server behavior?
- Does `UseOutproxyPlugin` represent a real registered local provider/plugin callback, and on exactly which tunnel families?

No implementation is authorized. If a non-HTTP cell lacks affirmative applicability, reclassify only when the evidence positively demonstrates non-applicability.

### WP3 — Re-freeze Streamr client residual applicability

Review every blocked Streamr client cell rather than assuming TCP equivalence.

At minimum review:

- `ConnectDelay`;
- `Profile`;
- `DelayOpen`;
- `Reduce`;
- `ReduceCount`;
- `ReduceTime`;
- `Close`;
- `CloseTime`;
- `NewDest`;
- any other blocked Streamr client cell mechanically present in the starting 96.

Determine whether the pinned Proposal/reference assigns each option to Streamr's datagram/session model and, if so, the exact runtime trigger. A missing meaningful datagram analogue is not sufficient by itself for `not_applicable`; affirmative reference evidence is required.

### WP4 — Re-freeze `Profile`

Resolve the exact semantics for each applicable client family.

Planning-time evidence suggests the Java Proposal-170 implementation maps `interactive` to a bounded `i2p.streaming.maxWindowSize` override rather than a router-global profile registry. Verify this against the pinned/reference stack.

Then inspect Emissary's existing neutral streaming configuration and answer:

- does a canonical `max_window_size`/streaming-profile owner already exist?
- is it consumed by the actual SAM streaming manager or only declared/dormant?
- what is the smallest neutral path that would make the setting operational?
- can all Proposal policy remain in I2PControl with only a generic streaming configuration seam below it?

M131 may correct blocker text/owner but may not wire the setting.

### WP5 — Re-freeze session activity, idle reduction, close and resume

Review the complete `Reduce`/`ReduceCount`/`ReduceTime` and `Close`/`CloseTime`/`NewDest` residual family against reference I2CP session semantics.

M121 remains authoritative that local TCP handler count is not equivalent to I2P-session activity. M131 must identify the exact lower-layer observation/control contract needed for a future implementation.

At minimum determine:

- which outbound and inbound application-message events reset reference session activity;
- whether streaming ACK/control/tunnel maintenance traffic counts or only client application send/receive activity;
- shared/subsession activity aggregation semantics;
- minimum/default idle durations and any exact lower bounds relevant to Proposal values;
- exact reduction behavior: target quantity, inbound/outbound scope, restoration trigger and repeated-idle behavior;
- exact close behavior and session destruction trigger;
- exact `NewDest` timing: successor identity generation only on resume after an idle close, and behavior on manual stop/start, network failure and cancelled/failed resume;
- whether Streamr participates in the same I2P-session idle owner.

Produce a candidate neutral capability contract, not code. The preferred shape to assess is:

1. generation/session-local application-activity observation;
2. bounded tunnel-pool quantity reduction/restoration control owned by the canonical pool manager;
3. typed session lifecycle/idle-close outcome observable by the existing I2PControl lifecycle owner without a Proposal-shaped core callback.

The future seam MUST NOT be based on local TCP socket count or per-byte I2PControl proxy bookkeeping if that differs from reference session semantics.

### WP6 — Re-freeze `UniqueLocalAddressPerClient`

Determine exact Java/I2PTunnel semantics and whether the feature can be implemented without relaxing literal-loopback target confinement.

Planning-time hypothesis to verify:

- trusted remote I2P identity deterministically selects a local source address;
- the server still connects only to the configured confined local target;
- the behavior may therefore be an I2PControl-owned source-bind policy rather than a router/LAN routing subsystem.

Required outputs:

- exact source-address derivation algorithm/range;
- IPv4/IPv6 behavior;
- socket bind/connect ordering and failure behavior;
- whether OS/platform restrictions make exact portable behavior impossible;
- path budget for a future I2PControl-only implementation if feasible;
- proof that no request-selected LAN/clearnet destination or DNS resolution is introduced.

### WP7 — Re-freeze `MultiHoming`

Verify whether Proposal/reference `MultiHoming` maps to `shouldBundleReplyInfo` and therefore controls periodic sender LeaseSet bundling rather than host-interface multihoming.

If verified, map the actual missing neutral primitives:

- Yosemite/session-option serialization or existing typed field consumption;
- SAM session-state retention in Emissary;
- outbound client-message LeaseSet bundling decision at the canonical message owner;
- no Proposal-specific API in core.

Identify exact future production paths and tests required. Do not implement them.

If the reference semantics are more complex than `shouldBundleReplyInfo`, retain blocked status and record the conflicting evidence explicitly.

### WP8 — Re-freeze `UseSSL`

Before any TLS capability work, determine the exact applicable tunnel families and directionality.

Separate all potentially confusable TLS meanings:

- I2PControl management HTTPS;
- Yosemite `SessionOptions.ssl` / SAM control-connection TLS;
- local client-facing TLS listener;
- local server-target TLS connector;
- HTTP CONNECT/SSL outproxy behavior.

For every current `UseSSL` blocked cell and every reference family where `UseSSL` may actually apply, record:

- exact role;
- certificate/key/trust ownership;
- whether TLS is terminated or originated locally;
- verification/SAN expectations;
- whether operator-supplied and/or managed identities are reference-compatible;
- restart/edit persistence requirements;
- fail-closed behavior;
- proposed future I2PControl-local path budget.

Do not reuse I2PControl management TLS material implicitly and do not propose certificate-verification bypasses.

M131 may reclassify applicability but may not add TLS runtime behavior.

### WP9 — Re-freeze `SigType`

M121 correctly established that Yosemite can serialize arbitrary `SIGNATURE_TYPE` but Emissary's private signing/generation path is Ed25519-only.

M131 must define the actual lower-layer work needed for configurable support:

- exact Proposal/reference accepted value set/spellings;
- which signature types are required for truthful Proposal behavior rather than optional reference extensions;
- destination certificate encoding requirements;
- private-key generation/import/serialization requirements;
- streaming/datagram signature generation/verification implications;
- persistent and transient destination compatibility;
- security/dependency implications of candidate Rust crypto implementations;
- whether one incremental additional signature type would constitute truthful configurable support or whether the full pinned value set is required.

No crypto implementation or dependency addition is authorized.

### WP10 — Re-freeze encrypted/authenticated LeaseSet residuals

Review `EncryptLeaseSet`, `OptionalLookup` and `LeaseSetClientAuths` for all blocked server families against:

- Proposal-170 parameter semantics;
- Java Proposal-170 mapping;
- Yosemite Y005 typed option validation/serialization;
- Emissary `LeaseSet2`, `DatabaseStore`, NetDB, destination/signing and publication paths.

Explicitly retire stale blocker language if Yosemite serialization is no longer missing. The residual blocker must name the first missing runtime owner below SAM.

Produce a dependency decomposition covering, where actually required:

- encrypted LeaseSet2 construction/parsing;
- blinding/signing key support;
- lookup-secret handling;
- DH/PSK client authorization;
- local secret/key generation and bounded persistence ownership;
- blinded/derived NetDB lookup/store keys;
- publication, lookup, decrypt/auth and restart lifecycle;
- interoperability evidence against a reference router.

The plan MUST distinguish legacy AES encrypted LeaseSet behavior from encrypted LS2/blinded modes instead of grouping all modes as one feature if the protocol owners differ.

No LeaseSet/NetDB/crypto production code is authorized.

### WP11 — Primitive clustering and future path budgets

After WP2-WP10, cluster the retained blocked cells by actual shared primitive rather than by historical milestone.

For each cluster record:

- exact cells;
- semantic contract;
- canonical owner;
- whether it is I2PControl-only, Yosemite/dependency-only, or requires a neutral Emissary lower-layer seam;
- smallest plausible production path set;
- dependency ordering;
- security hazards;
- deterministic unit/integration evidence;
- required live/reference interoperability evidence;
- stop conditions;
- whether the cluster is dependency-ready now.

Expected clusters to test, not assume:

- HTTP address-helper / SSL-outproxy behavior;
- local presentation TLS;
- streaming profile/window configuration;
- I2P-session activity + idle reduction/close/resume;
- per-client local source addressing;
- `shouldBundleReplyInfo`/LeaseSet bundling;
- outproxy-provider/plugin integration;
- destination signing-type generation;
- encrypted/authenticated LeaseSets.

If evidence shows different clustering, use the evidence rather than this list.

### WP12 — Ordered successor recommendation

Rank future M132+ work by:

1. semantic certainty;
2. number of cells sharing the primitive, as a secondary planning factor only;
3. containment cost;
4. security/crypto risk;
5. dependency readiness;
6. interoperability-test feasibility.

The likely high-value candidate is the neutral I2P-session activity/reduction/lifecycle primitive because many residuals share it, but M131 MUST NOT pre-authorize that conclusion. The closure must select the next handoff from evidence.

Only one next dependency-ready plan may be registered at M131 closure under the normal planning ceremony. Future dependent milestones may be named/outlined but remain unregistered.

## 7. Failure, cancellation, restart and contention analysis

M131 is planning/evidence-only, but the primitive architecture MUST explicitly reason about future runtime failure semantics.

For every future primitive cluster, record at least:

- allocation point and validation-before-effect boundary;
- cancellation owner;
- generation/restart ownership;
- lock scope and prohibition on holding locks across network/filesystem I/O, sleeps or joins;
- bounded task/timer/state requirements;
- rollback/old-generation preservation on failed edit/start;
- shared-session contention and final-member behavior where relevant;
- secret lifetime/zeroization/redaction requirements;
- behavior after partial lower-layer failure;
- no silent security downgrade or fallback.

A cluster lacking a credible failure/restart/ownership model is not dependency-ready even if its happy-path primitive appears simple.

## 8. Compatibility and migration

M131 itself has no runtime migration.

If cell applicability changes:

- historical closures remain unchanged;
- M095/M105/M110 and active docs record the new current authority;
- a previously blocked cell reclassified to `not_applicable` must not make old persisted unsupported configuration silently accepted;
- current fail-before-effect runtime behavior remains unchanged until a later implementation plan explicitly changes it.

No persisted schema, secret format, TLS material, destination key or tunnel definition is migrated by M131.

## 9. Evidence and test requirements

### 9.1 Reference evidence

Record exact immutable source identifiers for every external/reference conclusion where possible:

- Proposal revision/date;
- Proposal-170 Java PR head inspected;
- Java I2P/I2PTunnel source commit inspected;
- Yosemite exact dependency revision;
- Emissary baseline/head.

When Proposal text, Java Proposal implementation and current I2PTunnel runtime disagree, do not choose whichever reduces the matrix. Record the conflict and retain the safer blocked disposition unless the repository's existing authority hierarchy clearly resolves it.

### 9.2 Matrix guards

If M095/M105/M110 changes:

- mechanically recompute total cell counts;
- update exact matrix tests;
- add focused assertions for every changed cell and its evidence class;
- prove no cell changed to `apply`;
- prove all starting 96 cells are accounted for in the M131 evidence artifact.

### 9.3 Broad verification

Run at minimum from repository root:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --no-fail-fast
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

If no Rust/test fixture changes are required, the closure may explain why only the relevant planning/matrix guards were rerun, but M095/M105 must be executed if either artifact changes.

`cargo fmt --all -- --check` should be attempted only if Rust source/test files change. Existing stable/nightly formatter drift must be recorded rather than normalized through unrelated churn.

## 10. Authorized repository changes

M131 authorizes planning/evidence/documentation changes only.

Expected/allowed paths:

- `plans/implementation/i2pcontrol-proposal-170/131-residual-applicability-and-primitive-architecture-refreeze.md`;
- a new M131 machine-readable evidence artifact under `plans/implementation/i2pcontrol-proposal-170/` if useful;
- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml` only for evidence-backed `blocked_primitive` → `not_applicable` corrections, blocker-owner corrections, or metadata/current-head reconciliation;
- `plans/implementation/i2pcontrol-proposal-170/105-residual-option-audit.toml` for current residual reconciliation;
- `plans/implementation/i2pcontrol-proposal-170/110-completion-ledger.toml` for current-authority reconciliation;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/registry.md`;
- `docs/i2pcontrol/proposal-170-support.md` and `docs/i2pcontrol/tunnel-manager.md` only when current support/applicability wording changes;
- focused M095/M105 planning-guard tests only if needed to enforce changed evidence.

Forbidden production/dependency paths in M131 include:

- `emissary-core/src/**`;
- `emissary-util/src/**`;
- `emissary-cli/src/**` production code;
- any `Cargo.toml`/`Cargo.lock` dependency or feature change;
- `.github/**`, release/workflow/frontend/startup tunnel production paths;
- `eggstack/yosemite` writes.

If execution determines that a production code change is required to answer the audit, stop. Record it as a future primitive; do not broaden M131.

## 11. Required M131 evidence artifact

M131 closure SHOULD add a machine-readable artifact, for example:

- `plans/implementation/i2pcontrol-proposal-170/131-residual-primitive-map.toml`

or an equivalently reviewable format.

It MUST contain for every starting blocked cell:

- option;
- tunnel family;
- starting disposition;
- final M131 disposition;
- applicability evidence pointer;
- exact missing primitive if retained blocked;
- canonical owner;
- primitive cluster identifier;
- dependency readiness;
- security class;
- future path-budget identifier.

The artifact MUST additionally contain cluster records with path budgets and dependency edges so later plans cannot silently broaden lower-layer ownership.

## 12. Documentation/static guards

At M131 closure:

- registry identifies M131 closure and the one next registered handoff, if any;
- full-support roadmap records the corrected residual partition and dependency graph;
- implementation README identifies M131 as current residual-architecture authority while M130 remains the current implemented-subset runtime qualification authority;
- active docs retain **partial** Proposal-170 wording;
- historical M111-M130 closure files are not rewritten;
- M061/M062 containment artifacts are not modified unless a planning-only future path-budget record explicitly requires metadata clarification; no new lower-layer production path is accepted merely by mentioning it in M131.

## 13. Acceptance criteria

M131 may close only when all are true:

1. all 96 starting blocked cells are mechanically enumerated and individually accounted for;
2. every final blocked cell names the first real missing runtime primitive and canonical owner rather than stale parser/serializer or overly broad subsystem language;
3. every `not_applicable` correction has affirmative pinned/reference evidence;
4. zero cells are promoted to `apply`;
5. `UseSSL` exact applicability/direction is re-frozen;
6. `SSLProxies`, `JumpList` and `UseOutproxyPlugin` family applicability is re-frozen;
7. all Streamr residual applicability is explicitly reviewed;
8. `Profile` is mapped to its exact streaming/runtime semantic owner;
9. session activity/reduction/close/resume semantics and a neutral candidate primitive contract are frozen without using local TCP handler count as a substitute;
10. `UniqueLocalAddressPerClient` exact source-address semantics and confinement impact are frozen;
11. `MultiHoming` is either proven to be `shouldBundleReplyInfo`-based with an exact neutral primitive path, or retained blocked with conflicting evidence documented;
12. `SigType` lower-layer crypto/destination work is decomposed without adding crypto code;
13. encrypted/authenticated LeaseSet residuals are re-based on Yosemite Y005's actual wire capability and the first missing Emissary runtime owner;
14. primitive clusters have explicit future path budgets, failure/restart/security considerations and dependency readiness;
15. M095/M105/M110/docs are reconciled if the current authority changes;
16. no runtime production/dependency code changed;
17. broad planning/matrix verification passes with any pre-existing environmental/tooling drift recorded separately;
18. closure recommends/registers at most one dependency-ready M132+ handoff under the repository ceremony.

## 14. Stop conditions

Stop M131 and close with retained blockers rather than guessing if:

- the pinned Proposal/reference sources do not resolve applicability;
- an apparent simplification relies only on an unmerged Java implementation artifact that conflicts with the Proposal or actual I2PTunnel runtime;
- a cell would need approximation, accept-inert behavior or security downgrade;
- exact behavior depends on a broad router redesign with no neutral canonical owner;
- implementation would require a new unreviewed crypto/TLS/plugin dependency merely for matrix parity;
- the required behavior cannot be separated from unrelated router behavior without violating the containment boundary.

A retained blocker with an exact reason is a successful M131 result.

## 15. Closure evidence

The closure MUST record:

- exact execution head and production-behavior baseline;
- exact external/reference source revisions inspected;
- starting `284 / 96 / 460` counts;
- ending counts and exact changed-cell list, if any;
- the complete M131 primitive-map artifact;
- before/after blocker descriptions for every corrected owner;
- evidence for every `not_applicable` reclassification;
- zero `apply` promotions;
- the ordered future primitive dependency graph;
- exact candidate path budgets;
- security/containment review;
- verification command outcomes;
- one next registered dependency-ready handoff, or explicit no-handoff disposition;
- internal-only/upstream-read-only attestation.

M130 remains the runtime/security qualification authority for the implemented subset. M131 becomes the current authority only for residual applicability, blocker ownership and future primitive architecture.
