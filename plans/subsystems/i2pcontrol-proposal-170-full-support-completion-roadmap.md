# I2PControl Proposal 170 Full-Support Completion Roadmap

Status: active; M095-M096 and M098-M103 closed, M099 closed internally/partial, M097 and M104 closed as blocked, M105 closed; **M106 DelayOpen client-listener handoff is current**

Planning origin: M094 closed planning head `630a8fd1cd4e5943fcde0b5c16f5fc1e88b5d207`.

Current M105 audit baseline: `aa90c3afc830dcdbca8f6bf8acb5737acc73c366` — M104 closed as blocked.

Pinned external authority:

- I2P Proposal 170, `I2PControl Expansion`;
- status: `Open`;
- pinned revision: `2026-05-20`.

Canonical/internal authority:

- `plans/000-long-term-specification.md`;
- `plans/001-terminology-and-domain-model.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- ADR-0001/0002/0003/0004;
- M061/M062/M063 containment authority;
- M093 current tunnel security reclosure;
- M095 machine-readable full-support matrix;
- M097-M104 closure evidence.

## 1. Purpose

Move the internal fork from truthful partial Proposal 170 support to full support against the pinned `2026-05-20` revision while keeping Proposal 170 policy concentrated under `emissary-cli/src/i2pcontrol/**` and refusing to turn conformance gaps into broad router-core or dependency redesign.

The workstream is Proposal 170 only. It is not general I2PControl parity and it is not an upstream contribution program.

## 2. Current production state

The current production state at the M105 baseline is:

- RouterInfo: 43 canonical additions / 42 available / 1 protocol-permitted neutral / 0 unavailable;
- AddressBook CRUD, subscriptions, and all 13 `SetConfig` keys operational under the confined AddressBook owner;
- all 12 canonical TunnelManager tunnel types have real data planes;
- all 7 canonical TunnelManager actions are implemented;
- M097 applied `TunnelLength`, `TunnelQuantity`, and typed `EncType` where applicable;
- M098 applied the bounded client proxy/outproxy/auth/privacy subset;
- M099 applied the bounded server presentation/access/filter/admission/rate subset;
- all 6 ClientServicesInfo selectors are implemented;
- unsupported residual options fail before allocation rather than being persisted-and-ignored;
- full public-network/reference-router certification remains open;
- M104 closed as blocked because the final reviewed TunnelManager matrix still contains 164 applicable residual cells.

M104's reviewed matrix totals are:

- 70 canonical option rows × 12 canonical types = 840 cells;
- 218 `apply`;
- 164 applicable `blocked_primitive`;
- 458 `not_applicable`;
- 0 `planned_apply`, unsupported, unknown, or accept-inert cells.

M093 remains the current tunnel security authority. M088's accepted lower-layer pre-accept residual and the bounded Streamr availability limitation are not reopened merely for option parity.

## 3. Corrective findings through M104

### 3.1 M097 dependency finding

M097 proved that several common session/key options cannot be truthfully mapped through the currently accepted Yosemite/SAM path. It safely implemented the supported subset and closed as blocked rather than expanding the dependency/core boundary.

### 3.2 M098/M099 decomposition correction

The original M098/M099 graph treated successful M097 completion as a milestone-wide hard dependency. That was too coarse. Many client and server option cells already had exact owners inside existing I2PControl runtimes.

The corrected sequence therefore:

- let M098 implement exact client proxy/auth/privacy cells;
- let M099 implement exact server presentation/access/filter/admission/rate cells;
- transferred genuine missing-owner/session/key/LeaseSet cells to explicit residual ownership.

This preserved fail-closed semantics without using an external/library blocker to hold unrelated safe work hostage.

### 3.3 M104 residual finding

M104 performed the bounded integrated closure attempt and stopped correctly. It established that all ordinary planned implementation work had been consumed but 164 applicable cells remain blocked.

M104 did not determine whether every residual blocker is the narrowest final classification. The remaining question is now architectural/semantic rather than ordinary implementation completion:

- which cells are actually implementable entirely in existing I2PControl ownership;
- which need one minimal neutral canonical-owner seam;
- which are truly blocked by Yosemite/SAM/dependency capability;
- which require a material new architecture decision;
- which may be incorrectly classified as applicable for Emissary under the pinned/reference semantics;
- which remain semantically ambiguous.

M105 exists to answer that question without changing production behavior.

## 4. Ownership boundary

### Preferred ownership

Proposal 170 administrative/application/runtime policy belongs under:

`emissary-cli/src/i2pcontrol/**`

This includes option validation/application, proxy/filter policy, server admission configuration, administrative file confinement, matrix/audit reconciliation, and I2PControl interoperability harnesses.

### Lower-layer exception rule

A future production change outside `i2pcontrol` is allowed only when all are true:

1. the required behavior belongs to an existing lower-layer canonical owner;
2. no truthful I2PControl-local implementation exists;
3. exact paths/owners are named before implementation;
4. the seam is neutral/reusable rather than Proposal-170-shaped;
5. behavior is bounded and does not silently change unrelated router decisions;
6. M061 containment is not widened implicitly;
7. a registered successor plan explicitly authorizes it.

M102's existing network-error observation is the deliberate full-support lower-layer exception already accepted. M105 itself authorizes no new lower-layer production path.

### Dependency rule

No plan in this roadmap automatically authorizes:

- vendoring/forking/patching Yosemite;
- replacing Yosemite with an internal duplicate SAM stack;
- adding Proposal-170-shaped APIs to `emissary-core`;
- adding dependencies merely to make matrix counts green.

A residual option requiring one of those actions remains blocked until a separately approved architecture decision changes the boundary.

## 5. Invariants

- exact pinned names, types, actions, response shapes, and presence semantics;
- no fabricated state or inert accepted configuration;
- every applicable `apply` cell changes real runtime behavior;
- implementation difficulty is not evidence of `not_applicable`;
- a Java-specific implementation mechanism is not automatically an Emissary architecture requirement;
- Proposal 170 policy remains in I2PControl wherever possible;
- startup/control-plane ownership remains distinct;
- HTTP/IRC/Streamr anonymity and resource bounds remain mandatory;
- server local targets remain confined as accepted;
- direct I2P proxy traffic never falls through to clearnet DNS;
- clearnet proxy traffic requires an explicit I2P outproxy;
- secrets and path-valued configuration remain redacted/confined;
- LeaseSet security never silently downgrades;
- feature-disabled/default Emissary gains no I2PControl-only runtime/dependency behavior;
- external sources are read-only;
- all repository writes remain internal to `eggstack/emissary`.

## 6. Explicit non-goals

This workstream MUST NOT:

- implement unrelated base methods such as `GetKeys`, `GetRate`, `RouterManager`, `NetworkSetting`, or `AdvancedSettings`;
- add non-Proposal-170 methods, tunnel types, actions, statuses, or wire fields;
- redesign real tunnel data planes solely to share code;
- reopen M088/M091 without new independent security evidence;
- add DCC/WEBIRC/SOCKS BIND/UDP ASSOCIATE absent a pinned requirement;
- add router-wide banning merely for telemetry or server throttling;
- couple I2PControl to frontend state;
- add broad hosted CI/fuzz/coverage/release infrastructure;
- initiate or prepare upstream review/merge/submission/adoption/contact.

## 7. Target architecture

```text
                         Proposal 170 JSON-RPC
                                |
                                v
                  emissary-cli/src/i2pcontrol/**
      +-------------------------+--------------------------+
      |                         |                          |
      v                         v                          v
AddressBook config       Tunnel option policy       RouterInfo sources
and confined state       + existing real backends   + bounded local owners
      |                         |                          |
      v                         v                          v
existing downloader      existing Yosemite/SAM      existing neutral core
seam where required      interfaces only            observation seams only
```

No new Proposal 170 business object belongs in router core.

## 8. Full-support matrix rule

The M095 machine-readable matrix remains production-support authority.

Final closure requires:

- RouterInfo: all 43 rows available except proposal-permitted neutral semantics;
- AddressBook: all 13 SetConfig keys operational with explicit owners;
- TunnelManager: every canonical option/type cell is `apply` or evidenced `not_applicable`;
- zero applicable `planned_apply`, `blocked_primitive`, unsupported, or unknown cells;
- exactly 12 tunnel types and 7 actions;
- exactly 6 ClientServicesInfo selectors;
- compatibility aliases/extensions clearly separated from canonical completion.

Parser acceptance, persistence without runtime effect, fail-before-allocation rejection, or an M105 audit-candidate classification is not final `apply` evidence.

## 9. Current dependency graph

```text
M095 exact matrix + containment budget              [CLOSED]
  |
  +--> M096 AddressBook SetConfig                    [CLOSED]
  +--> M100 transit 15s                              [CLOSED]
  +--> M101 router news                              [CLOSED]
  +--> M102 network-error owner                      [CLOSED]
  +--> M103 banned-peer semantics                    [CLOSED]
  |
  +--> M097 common session/key options               [CLOSED AS BLOCKED]
         | supported: TunnelLength/TunnelQuantity/EncType
         | residual primitive blockers retained
         |
         +----------------------+----------------------+
                                |                      |
                                v                      |
M098 client/proxy/HTTP independent slice             |
[CLOSED]                                             |
  |                                                  |
  v                                                  |
M099 server/access/throttle independent slice        |
[CLOSED INTERNALLY — PARTIAL]                        |
  |                                                  |
  v                                                  |
M104 live interoperability + final reclosure         |
[CLOSED AS BLOCKED — 164 APPLICABLE RESIDUAL CELLS] |
  |                                                  |
  v                                                  |
M105 residual primitive/applicability audit          |
[CLOSED — SIX LOCAL TCP CLIENT CELLS IDENTIFIED]    |
  |                                                  |
  v                                                  |
M106 DelayOpen client-listener lifecycle             |
[READY — SIX TCP CLIENT FAMILIES; STREAMR EXCLUDED] |
```

## 10. Milestone status and exit conditions

### M095 — full-support matrix and containment budget

Status: closed.

Exit evidence remains the exact 43 / 13 / 12×options / 6 inventory and path budgets.

### M096 — AddressBook SetConfig

Status: closed.

All 13 pinned keys have bounded operational/metadata semantics. The sole non-I2PControl production amendment reused the existing AddressBook downloader seam rather than creating competing ownership.

### M097 — common tunnel session/key options

Status: closed as blocked.

Implemented safe subset:

- `TunnelLength`;
- `TunnelQuantity`;
- typed `EncType`.

Retained blockers include `Shared`, `UseSSL`, `TunnelVariance`, `TunnelBackupQuantity`, `SigType`, `CustomOptions`, `NewDest`, `PersistentClientKey`, and `PrivKeyFile` where current accepted ownership/serialization is absent.

M097's stop condition remains valid until M105 evidence says a narrower classification/path exists. M105 does not itself reopen M097 production work.

### M098 — client proxy, management, and HTTP independent slice

Status: closed.

Exact proxy/outproxy/auth/privacy behavior owned by existing I2PControl client runtimes is operational. Plugin/TLS-proxy/jump-list/client-management semantics without exact owners remain blocked.

### M099 — server access, throttle, and LeaseSet independent slice

Status: closed internally against pinned revision; partial.

Exact server presentation/access/filter/admission/rate behavior owned by accepted I2PControl server runtime is operational. LeaseSet/TLS/address-routing semantics without exact safe owners remain blocked.

### M100 — transit 15s

Status: closed.

Request-independent I2PControl-owned sampler over the authoritative cumulative transit source.

### M101 — router news

Status: closed.

Bounded signed router-news acquisition/cache using the existing SU3/certificate seam. Full public-network evidence remains part of a successful future reclosure.

### M102 — network error owner

Status: closed.

Minimal neutral v4/v6 observation in existing core owners; Proposal 170 mapping remains in I2PControl.

### M103 — banned peers

Status: closed.

Explicit by-design-empty router-wide banned-peer source after proof that Emissary has no such ban owner. No ban algorithm added.

### M104 — full live interoperability/reclosure

Status: closed as blocked.

Closure: `plans/closure/i2pcontrol-proposal-170/104-closure.md`.

M104 reached its final verification stop condition with 164 applicable `blocked_primitive` cells. It provides the baseline residual inventory for M105 and remains the authority that the repository cannot yet claim full support.

### M105 — residual TunnelManager primitive and applicability audit

Status: **closed**.

Plan:

`plans/implementation/i2pcontrol-proposal-170/105-residual-tunnel-option-primitive-audit.md`

M105 is an audit/infrastructure milestone with no production behavior. It must create:

`plans/implementation/i2pcontrol-proposal-170/105-residual-option-audit.toml`

covering exactly the 164 M104 residual cells.

For every cell, M105 must determine:

- pinned/reference applicability and exact required behavior;
- current Emissary owner/blocker;
- actual Yosemite/SAM wire capability where relevant;
- whether an exact existing I2PControl-local path exists;
- whether a minimal neutral existing-owner seam could suffice;
- whether support is dependency-blocked or requires a new architecture decision;
- whether the current applicability classification is probably wrong;
- security/anonymity/persistence/key/path implications.

Allowed audit dispositions are:

- `i2pcontrol_local_candidate`;
- `neutral_owner_candidate`;
- `dependency_blocked`;
- `architecture_decision_required`;
- `not_applicable_candidate`;
- `semantic_blocked`.

M105 does not change M095 production support counts. It must not implement options, patch/fork/vendor Yosemite, change dependencies, or widen core ownership.

Exit conditions:

- all 164 residual cells accounted for exactly once;
- evidence-backed applicability/semantics for every cell;
- exact candidate paths for any contained implementation candidate;
- exact missing wire/dependency behavior for dependency blockers;
- security/anonymity review for every relevant residual family;
- no production/dependency change;
- closure decides whether one bounded successor is dependency-ready;
- at most one successor implementation plan is registered after closure.

M105 closed with one dependency-ready successor: M106, limited to `DelayOpen`
for the six TCP-style client families. Streamr `DelayOpen` remains
`semantic_blocked`; the other 158 residual cells remain deferred, and all 164
cells remain `blocked_primitive` in the production matrix until later work
actually lands.

### M106 — DelayOpen client-listener lifecycle

Status: **ready; current handoff**.

Plan:

`plans/implementation/i2pcontrol-proposal-170/106-delay-open-client-listener.md`

M106 is a bounded I2PControl-local implementation slice for six TCP-style
client families. It does not authorize Streamr, Yosemite, core, util,
dependency, or full-support matrix changes beyond its eventual six-cell
reclosure.

## 11. M105 residual families

The audit baseline groups the 164 cells as:

| Family | Cells |
|---|---:|
| `Shared` | 7 |
| `UseSSL` | 4 |
| `TunnelVariance`, `TunnelBackupQuantity`, `SigType`, `CustomOptions` | 40 |
| `NewDest`, `PersistentClientKey` | 14 |
| `PrivKeyFile` | 10 |
| `UseOutproxyPlugin`, `SSLProxies`, `JumpList` | 12 |
| `ConnectDelay`, `Profile`, `DelayOpen`, `Reduce*`, `Close*` | 56 |
| `AllowInternalSSL`, `UniqueLocalAddressPerClient`, `MultiHoming` | 6 |
| `EncryptLeaseSet`, `OptionalLookup`, `LeaseSetClientAuths` | 15 |

M105 must evaluate the individual cells, not simply copy these family labels into a new ledger.

## 12. Security and anonymity requirements

Residual auditing and future option work must preserve:

- trusted Yosemite-derived peer identity;
- bounded transactional server admission;
- literal-loopback server targets;
- HTTP framing/spoof/fingerprint/Expect/POST protections;
- IRC registration/DCC/CTCP/lifetime protections;
- bounded Streamr subscriber/payload/fanout state;
- secret-safe persistence/get/logging;
- explicit I2P outproxy requirement for clearnet proxy traffic;
- no local DNS fallback for direct I2P destinations;
- no silent LeaseSet security downgrade;
- cancellation/edit/restart generation isolation;
- bounded shared-session/key ownership if such work is ever authorized.

An option path that achieves literal compatibility by weakening an accepted M093 anonymity or resource invariant is not dependency-ready.

## 13. Failure, lifecycle, and contention requirements

For future implementations:

- validate before allocation/publication where technically possible;
- failed edits preserve prior durable/runtime generations;
- per-name tunnel lifecycle remains serialized;
- timers/workers/tasks remain generation-local and cancellable;
- no lock crosses network I/O, sleeps, joins, or cancellation waits;
- bounded cleanup on stop/restart;
- blocked options continue to fail before allocation;
- no partial state presented as successful configuration.

For M105 specifically, no runtime changes are authorized. Partial/ambiguous audit evidence must remain visibly unresolved rather than being converted into a successor plan.

## 14. Verification policy

Use focused unit/integration/static guards plus the existing feature-gated containment/matrix suite. Do not build a CI farm for this phase.

M105 baseline commands:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment
git diff --check
```

If M105 adds a small dedicated audit coverage guard, run it explicitly. The guard should verify exact 164-cell coverage and disposition/evidence invariants only.

The historical `m063_feature_reachability` test target is absent in the current checkout; preserve that limitation rather than inventing unrelated replacement scope.

The repository-wide nightly/stable rustfmt mismatch is a tooling issue. Do not rewrite audited core files solely for formatter churn.

## 15. Risks and deferred work

### Residual Yosemite/SAM capability gaps

Risk: full support may remain blocked because the accepted dependency cannot express exact pinned session/LeaseSet semantics.

Response: M105 records exact missing wire capabilities. No automatic vendor/fork path follows.

### Applicability mistakes

Risk: some current blocked cells may reflect Java I2PTunnel implementation assumptions rather than true Proposal 170 applicability to a given Emissary tunnel family.

Response: M105 requires affirmative Proposal/reference evidence before recommending `not_applicable_candidate`. Difficulty alone is insufficient.

### Client identity/shared-session ownership

Risk: shared sessions or persistent/new client identities can create cross-tunnel ownership, identity rotation, secret persistence, and restart hazards.

Response: M105 must distinguish a bounded I2PControl control-plane owner from a router-wide destination/key subsystem. The latter requires an architecture decision.

### Proxy/TLS/plugin semantics

Risk: mechanically copying Java plugin/TLS machinery could introduce direct-clearnet fallback or trust bypass.

Response: audit contract behavior separately from Java implementation mechanism and preserve explicit I2P outproxy/trust boundaries.

### Server option security

Risk: TLS/local-address/multihoming options could weaken M093 target/anonymity boundaries.

Response: no request-selected LAN routing or unsafe address allocation is authorized by compatibility pressure.

### Live-network evidence

Risk: local verification may not have a functional I2P network/reference router.

Response: full support remains blocked rather than presenting process-local success as network interoperability.

## 16. Full-support exit condition

This roadmap closes only when a future M104 reattempt can truthfully record:

> Emissary fully supports I2P Proposal 170 against the pinned 2026-05-20 revision.

That claim requires zero applicable blocked/unsupported/planned TunnelManager cells and focused live interoperability. It does not mean general I2PControl parity or upstream acceptance.

M105 is a prerequisite decision audit; it cannot itself satisfy this exit condition.

## 17. Internal-only boundary

All work and writes remain internal to `eggstack/emissary`. External specifications, Yosemite, Java I2P, i2pd, I2P+, issues, commits, and pull requests are read-only evidence only.

No upstream issue/PR/review/submission/merge/adoption request, contribution preparation, branch/tag push, release, or maintainer contact is authorized by this roadmap.
