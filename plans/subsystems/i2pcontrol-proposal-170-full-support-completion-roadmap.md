# I2PControl Proposal 170 Full-Support Completion Roadmap

Status: active; M095-M096 and M100-M103 closed (M097 blocked); M104 remains blocked

Planning baseline: `630a8fd1cd4e5943fcde0b5c16f5fc1e88b5d207` — M094 closed planning head before this newly authorized phase.

Pinned external authority:

- I2P Proposal 170, `I2PControl Expansion`;
- status: `Open`;
- revision created/updated `2026-05-20`.

Canonical/internal authority:

- `plans/000-long-term-specification.md`;
- `plans/001-terminology-and-domain-model.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`;
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`;
- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`;
- `plans/adrs/ADR-0004-pinned-full-proposal-170-completion-boundary.md`;
- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml`;
- M093 production/security reclosure and M094 planning reconciliation.

## 1. Purpose

Move the internal fork from truthful partial Proposal 170 support to full support against the pinned `2026-05-20` revision without treating conformance count as authority to spread Proposal 170 policy into the heavily reviewed router core.

Current accepted baseline:

- RouterInfo: 43 canonical Proposal 170 additions, 42 available, 1 protocol-permitted neutral, 0 unavailable;
- AddressBook CRUD and subscription replacement operational;
- AddressBook `SetConfig` key inventory exact but non-empty configuration requests are currently rejected rather than applied;
- all twelve Proposal 170 TunnelManager types have real backends and all seven canonical actions are implemented;
- runtime option capability enforcement is fail-before-allocation, but many applicable Proposal 170 options are still explicitly rejected;
- ClientServicesInfo is operational for all six selectors;
- M061/M062/M063 containment authority remains accepted;
- M093 remains the current tunnel production/security authority;
- M088's lower-layer pre-accept residual and Streamr's bounded non-Sybil-resistant subscriber-set limitation remain accepted and are not reopened by completeness work unless a new direct defect is found.

This roadmap is a completeness workstream, not a general I2PControl or router-feature roadmap.

## 2. Ownership boundary

### 2.1 Preferred I2PControl ownership

All remaining administrative/application policy should live under:

`emissary-cli/src/i2pcontrol/**`

This includes:

- full-support contract/applicability/source matrices;
- AddressBook SetConfig parsing, persistence, path confinement, refresh/publish/proxy behavior, and administrative metadata;
- TunnelManager option normalization/capability/application semantics;
- request-independent 15-second transit sampling derived from authoritative existing counters;
- router-news acquisition/cache/parsing;
- mapping neutral network-error observations to Proposal 170 integer codes;
- final interoperability harness/control requests where they are specific to I2PControl.

### 2.2 Core-owner exception rule

A production edit outside `emissary-cli/src/i2pcontrol/**` is allowed only when all are true:

1. the required fact belongs to an existing lower-layer canonical owner;
2. no truthful I2PControl-local derivation is possible;
3. the plan names exact paths before implementation;
4. the exposed state is neutral and reusable, not named after Proposal 170 or JSON-RPC;
5. the change is bounded/passive and does not alter router decisions;
6. M061 containment is not widened implicitly.

M102 is the only currently planned milestone expected to require such a neutral owner change. M103 must avoid adding a peer-ban algorithm solely for a getter.

### 2.3 No aesthetic crate split

The current `i2pcontrol` feature/module boundary is the accepted isolation unit. Do not extract a standalone crate merely to make the source tree look cleaner.

## 3. Work classification

### Invariants

- exact pinned Proposal 170 names/types/actions/response shapes;
- truthful state; no fabricated zero/false/empty fallback;
- Proposal 170 policy stays in `i2pcontrol`;
- no new router algorithms solely for conformance;
- all lower-layer path changes explicitly budgeted;
- startup/control-plane ownership remains separate;
- existing HTTP/IRC/Streamr security boundaries remain mandatory;
- secrets and path-valued configuration remain confined/redacted;
- default/feature-disabled Emissary gains no I2PControl tasks or dependencies;
- external sources are read-only and all writes remain internal to `eggstack/emissary`.

### Capabilities

- all thirteen AddressBook SetConfig keys have operational semantics appropriate to Emissary;
- all applicable Proposal 170 TunnelManager option/type cells are implemented;
- all five currently unavailable RouterInfo rows become truthfully available;
- live interoperability establishes that all twelve tunnel families actually carry traffic and survive lifecycle/restart expectations;
- support documentation may truthfully claim full support against the pinned revision.

### Infrastructure

- machine-readable full-support matrix;
- versioned AddressBook configuration persistence/migration;
- bounded transit sampler;
- bounded news source/cache;
- neutral network-error snapshot if required;
- focused live interoperability harness.

### Polish

- support/conformance documentation reconciliation;
- diagnostics needed to make failure states understandable without leaking secrets.

## 4. Explicit non-goals

This roadmap MUST NOT:

- implement unrelated base I2PControl methods such as `GetKeys`, `GetRate`, `RouterManager`, `NetworkSetting`, or `AdvancedSettings`;
- add non-Proposal-170 methods, selectors, aliases, statuses, fields, tunnel types, or actions;
- redesign already-closed tunnel data planes merely to share code;
- reopen M088 lower-layer pre-accept work without a new independently demonstrated defect;
- add DCC, WEBIRC, SOCKS BIND/UDP ASSOCIATE, or other subfeatures unless the pinned Proposal 170 option matrix actually requires them;
- change runtime resolver precedence for administrative books except where the pinned SetConfig behavior explicitly requires an already-approved administrative file/source change;
- add frontend UI/state;
- create a router ban algorithm solely to satisfy `bannedpeers`;
- add broad core timers/pollers/news services;
- add hosted CI/fuzz/coverage/soak/release machinery solely for this phase;
- prepare or request upstream review, merge, issue, PR, adoption, submission, or maintainer contact.

## 5. Target architecture

```text
                           Proposal 170 JSON-RPC
                                  |
                                  v
                    emissary-cli/src/i2pcontrol/**
        +-------------------------+----------------------------+
        |                         |                            |
        v                         v                            v
 AddressBook config        Tunnel option mapping       RouterInfo completion
 state/runtime             + backend application        + news/transit local
        |                         |                      + neutral owner reads
        v                         v                            |
 existing address-book     Yosemite/SAM + existing            |
 runtime/persistence       bounded tunnel backends             |
                                                           only if unavoidable
                                                                v
                                             minimal neutral emissary-core owner
                                             (network-error state only expected)
```

No Proposal 170 field name, JSON type, or administrative configuration object should cross into core.

## 6. Full-support matrix rule

M095 created the authoritative machine-readable completion matrix. Final closure requires:

- all 43 Proposal 170 RouterInfo additions classified `available`, except a protocol-defined neutral value may remain neutral only where the proposal itself permits it;
- all 13 SetConfig keys classified with an operational owner and verification path;
- every TunnelManager canonical option cross-product cell classified `apply` or `not_applicable`; applicable `unsupported` cells are not allowed at final closure;
- all 12 tunnel types and 7 canonical actions retained exactly;
- all 6 ClientServicesInfo selectors retained exactly;
- compatibility aliases/extensions are clearly separated from the canonical contract and do not count toward completion.

An `available` or `apply` classification requires production evidence, not parser acceptance.

## 7. Dependency graph

```text
M095 exact full-support matrix + containment budget
  |
  +------------------+-------------------+------------------+
  |                  |                   |                  |
  v                  v                   v                  v
M096 AddressBook   M097 common tunnel  M100 transit 15s   M101 router news
SetConfig          session/key opts    source             source
                       |
                       +----------------------+
                       |                      |
                       v                      v
                 M098 client/proxy      M099 server/
                 management/HTTP opts   LeaseSet/access opts

M095 ----------------------------------------------+
  |                                                |
  v                                                v
M102 canonical network-error owner            M103 banned-peer semantic closure
  |                                                |
  +----------------------+-------------------------+
                         |
M097-M103 all closed ----+
                         |
                         v
                 M104 live interoperability +
                 full Proposal 170 reclosure
```

Dependency classes:

- M095 -> every implementation milestone: hard. No code work begins from an inferred option/source inventory.
- M097 -> M098/M099: hard for shared session/key option plumbing and final applicability model.
- M096 and M101 may proceed independently now that M095 is closed; M100 and M101 are now closed.
- M102 is closed with its bounded owner work and M103 is now closed with an explicit by-design-empty source because M095 recorded the completion budgets.
- M104 depends on all capability milestones M097-M103 closing; M096 is now closed.
- Public-network/reference-router reachability is an operational dependency for M104 only; local code may close earlier milestones without creating a hosted network farm.

Per planning governance, M095 was the only registered dependency-ready implementation plan at roadmap creation. M096, M097, and M100-M103 have now been processed; M097 is closed as blocked on named primitives. M098/M099 still wait for an unblocked M097, and M104 remains blocked on M097-M103.

## 8. Milestones and exit conditions

### M095 — Exact full-support matrix and containment budget

Plan: `095-full-support-contract-matrix-and-containment-budget.md`.

Status: closed; closure: `plans/closure/i2pcontrol-proposal-170/095-closure.md`.

Primary class: invariant / infrastructure.

Re-audit the pinned proposal against current production, create one machine-readable matrix covering RouterInfo, AddressBook SetConfig, TunnelManager options/type applicability, and ClientServicesInfo, and assign every remaining cell to an owner/path budget. This milestone made no runtime behavior change. The resulting authority is `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`, guarded by `emissary-cli/tests/m095_full_support_matrix.rs`.

Exit: no unknown applicability/source row remains; all later milestones have exact bounded inputs; lower-layer candidates are explicitly identified before code. M095, M096, and M100-M103 are closed; M097 is closed as blocked with exact Yosemite/SAM and key-lifecycle blockers; M098/M099 remain blocked on M097 and M104 remains blocked on M097-M103.

### M096 — AddressBook SetConfig operational completion

Plan: `096-addressbook-setconfig-operational-completion.md`.

Status: closed; closure: `plans/closure/i2pcontrol-proposal-170/096-closure.md`.

Primary class: capability / security / persistence.

Implement all thirteen pinned SetConfig keys with deterministic persistence and security-preserving path semantics. Behaviorally meaningful keys must affect the active AddressBook runtime; harmless UI metadata may round-trip without creating frontend coupling.

Exit: non-empty valid SetConfig requests succeed only after durable/runtime publication; restart preserves semantics; invalid/unsafe paths fail before mutation.

### M097 — Common tunnel session/key/profile option completion

Plan: `097-tunnel-common-session-and-key-option-completion.md`.

Status: blocked; closed as blocked by missing Yosemite/SAM and key-lifecycle primitives.

Primary class: capability / infrastructure.

Implement common Proposal 170 options that map to destination/session/tunnel construction and key persistence through existing Yosemite/SAM/backend primitives, including exact handling of tunnel length/variance/quantity/backup, signing/encryption types, shared/persistent/new-destination semantics, CustomOptions policy, and confined private-key references where applicable.

Exit: the common option matrix has runtime evidence for length/quantity/encryption cells; variance, backup, SSL, signing, custom, shared, persistent, new-destination, and private-key import cells remain explicitly blocked on named primitives; no new core API is introduced for convenience.

### M098 — Client/proxy/management/HTTP option completion

Plan: `098-client-proxy-management-and-http-option-completion.md`.

Status: blocked on M097.

Primary class: capability / security.

Complete client proxy/outproxy/authentication, client management, and HTTP-client filter option semantics across the relevant real backends.

Exit: every applicable client-side cell is applied; unsafe proxy exposure and secret handling remain fail-closed.

### M099 — Server/access/LeaseSet option completion

Plan: `099-server-access-throttle-and-leaseset-option-completion.md`.

Status: blocked on M097.

Primary class: capability / security.

Complete server access/throttle/filter/host controls and LeaseSet encryption/authentication/lookup semantics without weakening M074-M093 server admission/filtering boundaries.

Exit: every applicable server-side cell is applied and verified before allocation/publication.

### M100 — Request-independent transit 15-second source

Plan: `100-routerinfo-transit-15s-source-completion.md`.

Status: closed; dependency M095 closed.

Primary class: capability / infrastructure.

Implement a feature-gated I2PControl-owned sampler over the existing authoritative cumulative transit counter. Sampling history must exist independently of API calls.

Exit: deterministic tests prove request-independent 15-second bytes/sec semantics and feature-off zero impact. M100 is closed with the required bounded sampler and source disposition evidence.

### M101 — Router news source

Plan: `101-routerinfo-news-source-completion.md`.

Status: closed; dependency M095 closed; closure:
`plans/closure/i2pcontrol-proposal-170/101-closure.md`.

Primary class: capability / security / operations.

Implement the pinned router-news semantics in I2PControl with bounded source acquisition, authenticity/format validation, caching, refresh, cancellation, and failure behavior. No core news subsystem.

Exit: `i2p.router.news` returns real bounded news content from the adopted source and remains truthful during source failure/restart.

### M102 — Canonical IPv4/IPv6 network-error owner

Plan: `102-routerinfo-network-error-owner-completion.md`.

Status: closed; M095 owner/path audit closed; closure: `plans/closure/i2pcontrol-proposal-170/102-closure.md`.

Primary class: capability / containment.

Add the smallest neutral state necessary for explicit IPv4/IPv6 network-error reasons only if current lower-layer owners cannot already provide them. Wire-code mapping remains under I2PControl.

Exit: both error rows come from explicit canonical state; no transport/router decision or algorithm changes; exact core path delta stays within the plan budget. M102 is closed with uninitialized/firewalled states remaining truthfully unavailable.

### M103 — Banned-peer semantic closure

Plan: `103-routerinfo-banned-peer-semantic-completion.md`.

Status: ready; M095 semantic/source audit closed.

Primary class: capability / invariant.

Determine and implement the truthful banned-peer result without creating a ban engine merely for telemetry. Prefer read-only exposure of an existing enforced exclusion owner. If exhaustive evidence proves Emissary has no possible banned state, codify a by-design empty result with static/runtime proof rather than an unowned fallback.

Exit: `bannedpeers` has an authoritative by-design-empty semantic owner; no ban
behavior or ban-management API is introduced, and M104 remains blocked on M097.

### M104 — Live interoperability and full-support reclosure

Plan: `104-full-proposal-170-live-interoperability-and-reclosure.md`.

Status: blocked on M097-M103; M103 is closed, but M097 remains blocked.

Primary class: invariant / capability / operations.

Reconcile the full matrix, run focused real-network/reference-router interoperability, verify persistence/restart and all twelve tunnel families, review containment/security, update support documentation, and decide whether the pinned full-support claim is justified.

Exit: every pinned applicable matrix cell is evidenced; no high/medium correctness/security/containment finding remains; docs state full support only if evidence supports it.

## 9. Cross-cutting failure/lifecycle requirements

- Administrative mutations are atomic/durable before success where the existing control-plane contract requires durability.
- Background sampler/news tasks are bounded, cancellable, and created only for enabled I2PControl composition.
- A source acquisition failure does not corrupt the prior good generation or fabricate a fresh empty/zero value.
- Tunnel option validation occurs before listener/session/target allocation.
- Restart reconstructs configuration and backend-owned identity from durable state without adopting startup-managed resources.
- No lock is held across network I/O, sleeps, joins, or cancellation waits where avoidable.
- Concurrent edits/lifecycle operations preserve current generation/name ownership semantics.

## 10. Security requirements

- Preserve M093's accepted production/security conclusions unless new evidence directly invalidates one.
- Keep HTTP/IRC filtering non-bypassable.
- Keep server local targets literal-loopback/confined as currently accepted.
- Do not weaken trusted peer identity or admission accounting for option parity.
- SetConfig path values cannot grant arbitrary filesystem access.
- News content is untrusted input and must be bounded/validated before retention/serialization.
- Proxy/key/auth secrets never appear in logs, errors, Debug, support matrices, or ordinary API responses.
- Core network-error observations are passive and Proposal-170-agnostic.
- No new peer-ban behavior is introduced solely for telemetry.

## 11. Compatibility and migration

- Public canonical Proposal 170 spelling remains unchanged.
- Existing compatibility aliases/extensions remain optional internal compatibility and do not redefine the canonical matrix.
- Tunnel definition schema changes should be additive/versioned only if new persisted option representation is unavoidable.
- AddressBook configuration persistence must define migration from the current state where non-empty SetConfig was rejected and therefore no legacy accepted-but-inert config should exist.
- Feature-disabled/default builds must remain behaviorally unchanged.

## 12. Verification discipline

Each milestone uses focused package/integration tests plus the existing containment guards. Do not add a new global CI regime merely for this roadmap.

Common commands, where relevant:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m063_feature_reachability
git diff --check
```

Core tests are required only for milestones that actually modify core. M104 adds focused live interoperability; it does not require turning that environment into permanent hosted CI.

Existing nightly/stable rustfmt drift documented by M090-M094 must not cause formatter-only churn across audited core files. Formatting verification should use the repository's accepted toolchain policy; do not widen production diffs merely to satisfy an unrelated formatter version.

## 13. Documentation and closure rule

`docs/i2pcontrol/proposal-170-support.md` must remain truthful after every milestone. Until M104 closes successfully, the top-level status remains partial support.

Every milestone receives a closure record under `plans/closure/i2pcontrol-proposal-170/`. A code commit or green test suite alone is not closure.

M104 may use the final statement only if the matrix and live evidence justify it:

> Emissary fully supports I2P Proposal 170 against the pinned 2026-05-20 revision.

Because the proposal is open, any later upstream revision requires a new delta audit.

## 14. Risks and mitigations

| Risk | Mitigation |
|---|---|
| Full-support goal becomes excuse for broad core changes | local-first dependency order; ADR-0004; exact path budgets |
| Option matrix hides unsupported behavior under parsing/round-trip | M095 applicability matrix; APPLY requires runtime evidence |
| SetConfig creates arbitrary filesystem authority | confined administrative root, normalization, symlink/escape checks |
| Transit metric repeats M049 request-history defect | dedicated periodic sampler independent of requests |
| News fetch becomes privacy/resource hazard | bounded source/cache, explicit cadence, cancellation, failure semantics |
| Network error is fabricated from reachability adjacency | explicit neutral canonical state only |
| Ban getter causes new routing behavior | M103 forbids a ban algorithm solely for telemetry |
| Security hardening regresses during option parity | M093 invariants retained; focused negative tests |
| Proposal changes while implementation is underway | pinned 2026-05-20 contract and later delta audit |
| Planning scope drifts into base I2PControl/upstream work | explicit non-goals and internal-only closure attestation |

## 15. Internal-only rule

All implementation, planning, closure, and repository writes are internal to `eggstack/emissary`.

External specifications and repositories are read-only evidence. No milestone authorizes an upstream issue, pull request, review request, merge request, contribution package, submission checklist, adoption request, branch/tag push, release, or maintainer contact.
