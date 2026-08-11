# I2PControl Proposal 170 Containment Corrective Roadmap

Status: active; M060 ready

Planning baseline: `adb2f52543764b267b2bcb282d093111001ae4b2` — merged M057 closure head

Upstream comparison baseline: `eepnet/emissary@9b43484a21d5a1291c4881cdae62a36c527f8c0f` — current upstream `master` and fork merge base at planning time

Source workstream:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` — source-completion/truthfulness work through M057;
- accepted RouterInfo disposition remains 43 total / 37 available / 1 protocol-permitted neutral / 5 unavailable.

Canonical and governance references:

- `plans/000-long-term-specification.md`;
- `plans/001-terminology-and-domain-model.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`;
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`;
- `plans/closure/i2pcontrol-proposal-170/037-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/056-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/057-closure.md`.

Pinned external contract:

- I2P Proposal 170 `I2PControl Expansion`, revision `2026-05-20` as already pinned by the accepted workstream;
- external sources remain read-only and do not authorize upstream interaction.

## 1. Purpose and ownership boundary

The Proposal 170 implementation is operational for its supported surface and now keeps most control-plane policy under `emissary-cli/src/i2pcontrol/**`. The remaining concern is security-review contamination of the original Emissary codebase: the fork differs from upstream across a broad set of original CLI and `emissary-core` paths because later RouterInfo, ClientServicesInfo, AddressBook, tunnel-runtime, logging, and observation work required access to authoritative runtime facts.

This roadmap performs one bounded containment corrective sequence. It does **not** reopen Proposal 170 source completeness, add unsupported tunnel types, or attempt to make the five truthfully unavailable RouterInfo additions available. Its purpose is to reduce and then prove the minimum non-`i2pcontrol` production delta required to preserve the already accepted behavior.

The governing ownership rule is asymmetric:

- `emissary-cli/src/i2pcontrol/**` owns Proposal 170/I2PControl method semantics, JSON-RPC types, selectors, compatibility behavior, validation, bounds, administrative persistence, runtime aggregation, error mapping, and support disposition.
- Existing CLI/runtime modules may retain only narrow feature/configuration composition and neutral capability/observation adapters that they uniquely own.
- `emissary-core` may expose only neutral, bounded, passive, read-only facts that cannot be truthfully obtained above the canonical owner.
- No original router/protocol path may interpret a fact according to Proposal 170 or depend on JSON-RPC/control-plane types.

The desired end state is not zero changes outside `i2pcontrol`; that would be artificial and may force duplicate state or incorrect observation. The desired end state is **zero unjustified changes outside `i2pcontrol`**, with every retained change small, neutral, owner-local, and statically recorded.

## 2. Work classification

### Invariants

- Supported Proposal 170 behavior, exact wire spelling/types, authentication/TLS behavior, persistence semantics, AddressBook owner coherence, ClientServicesInfo truthfulness, and supported client/server tunnel lifecycle remain unchanged.
- The accepted RouterInfo matrix remains 37 available / 1 neutral / 5 unavailable.
- Unsupported tunnel data planes remain unsupported and resource-free.
- Default/no-I2PControl router behavior must not gain I2PControl-specific tasks, probes, sampling loops, state authorities, or network behavior.
- No router, peer-selection, NetDB, tunnel construction/routing, transport handshake/retransmission/congestion, cryptographic, LeaseSet, or I2NP algorithm change is authorized.
- No upstream write/review/submission activity is authorized.

### Infrastructure

- A complete fork-delta containment ledger against the pinned upstream merge base.
- Narrow original-CLI adapters where needed.
- A consolidated neutral core inspection/observation boundary where owner-local observation is unavoidable.
- A current machine-readable containment manifest and static guards.

### Polish

- Reverting obsolete hooks, duplicate helpers, stale scaffolding, unnecessary imports/exports, and Proposal-170-derived policy from original modules.
- Documentation that explains every retained non-`i2pcontrol` path.

### Capabilities

No new external capability is created by this roadmap. Capability behavior is frozen and used only as regression evidence.

## 3. Explicit non-goals

This work must not:

- implement news, banned-peer ownership, transit 15-second sampling, v4/v6 network-error ownership, or any other currently unavailable RouterInfo source;
- implement HTTP server/bidirectional server, IRC, SOCKS-IRC, CONNECT, Streamr, or any other missing tunnel data plane;
- add new I2PControl methods, aliases, selectors, statuses, fields, or compatibility extensions;
- extract the implementation into a new Cargo crate merely for aesthetic isolation;
- perform a repository-wide event-bus, observer-framework, service-registry, or configuration refactor;
- alter startup task adoption/control or unrelated normal Emissary CLI behavior;
- add CI, release automation, coverage, fuzz, soak, platform matrices, or generated evidence bundles;
- prepare or seek an upstream merge, review, issue, pull request, proposal update, or maintainer contact.

## 4. Current state

At baseline `adb2f525`, the fork is 255 commits ahead of upstream merge base `9b43484a`. The compare surface shows Proposal-170-related additions concentrated under `emissary-cli/src/i2pcontrol/**`, but original production paths also differ in these major groups:

- original CLI integration: `emissary-cli/src/address_book.rs`, `config.rs`, `main.rs`, `lib.rs`, `logger.rs`, proxy HTTP/SOCKS paths, and client/server tunnel paths;
- core inspection/router plumbing: `error/mod.rs`, `events.rs`, `inspection.rs`, `lib.rs`, `router/context.rs`, `router/mod.rs`, `runtime/mod.rs`, `subsystem/mod.rs`, `primitives/router_identity.rs`;
- client-service observation: `i2cp/socket.rs` and several SAM parser/session/socket/streaming lifecycle paths;
- transport observation: `transport/mod.rs`, NTCP2 module/session paths, and multiple SSU2 message/peer-test/relay/session/socket paths;
- tunnel observation: `tunnel/mod.rs`, `tunnel/pool/mod.rs`, and `tunnel/transit/mod.rs`;
- build/configuration files required by the feature and dependencies.

M037 previously reduced AddressBook/SAM leakage and established a static boundary, but later M045–M055 RouterInfo source work necessarily expanded the live observation surface. The historical M037 manifest therefore remains historical evidence, not the final current containment authority.

The current concern is not evidence of router-algorithm contamination; it is that the physical review delta is broader than desirable. This roadmap must distinguish unavoidable owner-local observation from removable implementation residue before editing audited code.

## 5. Target architecture

### 5.1 I2PControl owner

All administrative/control policy remains in `emissary-cli/src/i2pcontrol/**`, including:

- Proposal 170 and base I2PControl request/response models;
- source availability/unavailability decisions;
- AddressBook administrative DTOs, generations, persistence, repair/import policy, and subscriptions;
- TunnelManager definitions, persistence, backend registry, lifecycle coordination, and unsupported backend semantics;
- ClientServicesInfo aggregation and bounds;
- RouterInfo aggregation, sorting, joining, numeric/Base64 mapping, and sanitized errors;
- authentication, rate limiting, server/TLS behavior, and control-plane observability.

### 5.2 Original CLI boundary

Original CLI modules may retain only:

- feature-gated configuration values needed to enable/bind I2PControl;
- startup/shutdown composition of already-defined `i2pcontrol` services;
- narrow neutral callback/observer/owner interfaces where the original runtime owns the authoritative fact or lifecycle event;
- legacy AddressBook behavior required independently of I2PControl.

They must not own Proposal 170 persistence schemas, JSON-RPC semantics, selector policy, support classification, or duplicate control-plane state.

### 5.3 Core boundary

Core should converge on the smallest practical neutral surface, preferably centered on generic inspection DTOs/handles and owner-local passive notifications. A retained protocol-path modification must satisfy all of the following:

1. the fact is authoritative only at that owner;
2. collecting it at a higher already-modified owner would lose truth, ordering, or required bounds;
3. the hook does not change protocol decisions or timing semantics;
4. the hook is cheap/no-op when unused;
5. no live socket, mutable owner, secret material, message payload, or command channel crosses the seam;
6. no Proposal 170/I2PControl terminology or wire policy exists in the core path.

No general event framework is required. Prefer deletion and consolidation over abstraction growth.

## 6. Dependency graph

```text
M057 closed source/truthfulness planning state
   |
   v
M058 — non-i2pcontrol fork-delta inventory and containment ledger — closed
   |
   v
M059 — original CLI/runtime adapter containment — closed; M058 closed
   |
   v
M060 — core observation seam consolidation — ready; M059 closed
   |
   v
M061 — independent containment reclosure and static-guard refresh — planned; hard-blocked on M060
```

M060 is dependency-ready in the active registry. M061 remains planned until M060 closes.

## 7. Milestones

### M058 — Non-I2PControl fork-delta inventory and containment ledger

Class: infrastructure/corrective audit.

Objective: freeze the current upstream/fork delta and classify every changed production path outside `emissary-cli/src/i2pcontrol/**` before any further audited-code edits.

Deliverables:

- machine-readable path ledger with exact upstream/fork baselines;
- classification and owner/rationale for every non-`i2pcontrol` production path;
- candidate-removal and candidate-consolidation sets for M059/M060;
- zero production behavior changes.

Exit: every current non-`i2pcontrol` production delta is classified and M059 receives a bounded original-CLI path budget.

### M059 — Original CLI/runtime adapter containment

Class: corrective implementation.

Objective: move remaining Proposal-170-derived policy/aggregation out of original CLI/runtime files, revert unnecessary original-CLI changes, and leave only minimal configuration/composition or neutral runtime adapters.

Primary target groups: AddressBook, config/main/lib/logger, proxy HTTP/SOCKS, and existing client/server tunnel modules.

Exit: no original CLI/runtime file owns Proposal 170 wire/persistence/support policy; every retained delta has a direct runtime-owner justification; behavior remains regression-equivalent.

### M060 — Core observation seam consolidation

Class: corrective implementation/security containment.

Objective: reduce and consolidate the modified `emissary-core` review surface without weakening authoritative observations used by the supported control-plane API.

Primary target groups: inspection/router plumbing, SAM/I2CP observations, transport peer/status/stat observations, and tunnel-pool/transit observations.

Exit: obsolete/duplicate core hooks are removed, higher-level aggregation is used where semantically equivalent, and every retained protocol-owner hook has explicit necessity evidence. No new core path may be introduced without stopping for replanning.

### M061 — Independent containment reclosure and static-guard refresh

Class: invariant/closure.

Objective: perform a production-free independent review of the final diff, install current static containment guards, and record the final retained non-`i2pcontrol` boundary.

Exit: supported behavior passes focused regression, default/no-feature behavior remains unchanged, every retained path is documented, the current manifest is enforced, and no medium/high containment or behavior defect remains.

## 8. Cross-cutting requirements

### Storage and migration

No persistence schema migration is authorized. Existing AddressBook, tunnel-definition, server-secret, and other accepted state must remain readable and semantically identical. Moving code ownership must not create dual writers.

### Protocol and compatibility

No wire contract change. Exact Proposal 170/base compatibility semantics remain those accepted before this roadmap. Unavailable selectors remain unavailable rather than being approximated.

### Security and authorization

No new mutable authority crosses module boundaries. Authentication/TLS/token handling remains inside I2PControl. Core/original runtime hooks must be passive and non-sensitive. The work must decrease or hold constant the security-review surface; it must never broaden it for convenience.

### Concurrency, cancellation, restart, and recovery

Containment movement must preserve existing task ownership and shutdown behavior. Passive hooks cannot own router task cancellation. Observer failure must not alter protocol lifecycle. Existing incomplete/fail-closed observation semantics remain. No locks across `.await`, sleep, network I/O, filesystem enumeration, or JSON serialization are introduced.

### Performance and resource use

No new polling task, background sampler, persistent metric store, unbounded channel/map, or general event bus. The absent-observer/default path should remain allocation-free or equivalent to the accepted baseline wherever practical.

### Documentation and operations

The final documentation must distinguish I2PControl policy from generic runtime seams and give a path-by-path rationale for the retained upstream delta. No deployment or release-process expansion is part of this work.

## 9. Verification strategy

Verification remains local and proportional.

M058 is planning/audit-only: compare, path inventory, static searches, and `git diff --check`.

M059 focuses on `emissary-cli` no-feature and `i2pcontrol` feature tests, AddressBook, tunnel lifecycle, production composition, ClientServicesInfo, and containment tests.

M060 adds focused `emissary-core` SAM/I2CP, transport, tunnel, and inspection tests plus the `emissary-cli` feature regressions that consume those observations. Do not create an expanded CI matrix.

M061 reruns the bounded integrated package checks required to prove behavior preservation and static containment. Hosted CI is not required unless a pre-existing platform-specific behavior cannot be tested locally; no new workflow is authorized.

## 10. Risks and decision points

| Risk | Decision/mitigation |
|---|---|
| Chasing zero diff creates duplicate or stale state | retain the smallest canonical-owner seam and document why |
| Consolidating transport/tunnel hooks changes metric truth | compare old/new observation fixtures; stop rather than approximate |
| AddressBook movement creates a second owner | preserve single publication/persistence authority; no schema migration |
| Generic abstraction becomes larger than direct hooks | prefer small owner-local callbacks/inspection methods; no framework rewrite |
| Removal breaks disabled/default mode | mandatory no-feature and runtime-disabled regressions |
| Static manifest becomes historical/stale again | M061 manifest describes current retained boundary and is directly exercised by tests |
| Work drifts into the five unavailable RouterInfo rows | final 37/1/5 matrix is invariant; any source-completion proposal is a blocker |
| Upstream interaction is inferred from comparing upstream | upstream access is read-only only; no contribution activity |

## 11. Completion definition

This corrective roadmap is complete only when:

- every non-`i2pcontrol` production delta from the pinned baseline has a recorded classification;
- unnecessary original CLI/core changes are removed or reverted;
- remaining original runtime/core changes are minimal neutral seams with exact owner justification;
- no Proposal 170 wire/admin/support policy exists outside `emissary-cli/src/i2pcontrol/**` except unavoidable feature/configuration naming at composition boundaries;
- accepted Proposal 170 operational behavior and the 37/1/5 RouterInfo disposition are unchanged;
- default/no-feature Emissary behavior is regression-equivalent;
- current static guards enforce the final boundary;
- no new unsupported capability, CI/release machinery, or upstream interaction was introduced.

The completion criterion is **minimum justified delta**, not a predetermined number of modified files.

## 12. Milestone status

| Milestone | Status | Implementation plan | Closure record | Blockers |
|---|---|---|---|---|
| M058 | closed | `plans/implementation/i2pcontrol-proposal-170/058-non-i2pcontrol-delta-inventory.md` | `plans/closure/i2pcontrol-proposal-170/058-closure.md` | audit-only; 47 paths classified; no production changes |
| M059 | closed | `plans/implementation/i2pcontrol-proposal-170/059-cli-runtime-containment.md` | `plans/closure/i2pcontrol-proposal-170/059-closure.md` | exact original-CLI budget implemented; no core changes |
| M060 | ready | `plans/implementation/i2pcontrol-proposal-170/060-core-observation-containment.md` | to be created after implementation | M059 closure accepted; exact core budget frozen |
| M061 | planned | `plans/implementation/i2pcontrol-proposal-170/061-containment-reclosure.md` | to be created after implementation | hard: M060 closure |

M051 from the source-completion roadmap remains independently blocked by absent substantive news/ban owners and is not a dependency of this containment sequence.
