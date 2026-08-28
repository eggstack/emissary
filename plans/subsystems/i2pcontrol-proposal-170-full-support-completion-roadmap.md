# I2PControl Proposal 170 Full-Support Completion Roadmap

Status: active; M095-M096, M098, and M100-M103 closed; M097 closed as blocked; M099 is current handoff; M104 remains blocked

Planning origin: M094 closed planning head `630a8fd1cd4e5943fcde0b5c16f5fc1e88b5d207`.

Current corrective planning baseline: post-M103 `master` (`30cd8bcc9728c286b418cfb534d4f19c6b1eb4f5`).

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
- M095 machine-readable full-support matrix.

## 1. Purpose

Move the internal fork from truthful partial Proposal 170 support to full support against the pinned `2026-05-20` revision while keeping Proposal 170 policy concentrated under `emissary-cli/src/i2pcontrol/**` and refusing to turn conformance gaps into broad router-core redesign.

The workstream is Proposal 170 only. It is not general I2PControl parity and it is not an upstream contribution program.

## 2. Current production state

The current production state after M103 is:

- RouterInfo: 43 canonical additions / 42 available / 1 protocol-permitted neutral / 0 unavailable;
- AddressBook CRUD, `SetSubscriptions`, and all 13 `SetConfig` keys operational under the confined AddressBook owner;
- all 12 canonical TunnelManager tunnel types have real data planes;
- all 7 canonical TunnelManager actions are implemented;
- M097 applied `TunnelLength`, `TunnelQuantity`, and typed `EncType` where applicable;
- unresolved common/session/key options still fail before allocation rather than being persisted-and-ignored;
- all 6 ClientServicesInfo selectors are implemented;
- full public-network/reference-router certification remains open;
- full support remains blocked by TunnelManager option parity and M104 live reclosure.

M093 remains the current tunnel security authority. M088's accepted lower-layer pre-accept residual and the bounded Streamr availability limitation are not reopened merely for option parity.

## 3. Corrective dependency finding after M097

The original roadmap treated M098 and M099 as milestone-wide hard dependents of successful M097 completion.

M097 closure proved that decomposition too coarse:

- some common/session/key cells genuinely require missing shared-session, destination/key, private-key-import, or Yosemite SAM serializer primitives;
- many client proxy/outproxy/auth/HTTP-filter options are already owned by bounded I2PControl client runtimes;
- many server HTTP presentation/access/filter/admission/rate options are already owned by the accepted M074-M093 I2PControl server runtime.

Therefore dependency authority is now per option cell, not per entire milestone.

This correction does **not** redefine full support. Residual blocked cells still prevent M104.

## 4. Ownership boundary

### Preferred ownership

Remaining Proposal 170 policy belongs under:

`emissary-cli/src/i2pcontrol/**`

This includes option validation/application, proxy/filter policy, server admission configuration, administrative file confinement, matrix reconciliation, and final I2PControl interoperability harnesses.

### Lower-layer exception rule

A change outside `i2pcontrol` is allowed only when:

1. the required fact belongs to an existing lower-layer canonical owner;
2. no truthful I2PControl-local implementation exists;
3. exact paths/owners are named before implementation;
4. exposed state is neutral/reusable rather than Proposal-170-shaped;
5. behavior is bounded/passive and does not alter router decisions;
6. M061 containment is not widened implicitly.

M102's existing network-error observation is the deliberate full-support lower-layer exception. The corrected M098/M099 passes authorize no new core path.

### Dependency rule

No plan in this roadmap authorizes:

- vendoring/forking/patching Yosemite;
- replacing Yosemite with an internal duplicate SAM stack;
- adding Proposal-170-shaped APIs to `emissary-core`;
- adding dependencies merely to make matrix counts green.

If a residual option requires one of those actions, it remains blocked until a separately approved architecture decision changes the boundary.

## 5. Invariants

- exact pinned names, types, actions, response shapes, and direct presence semantics;
- no fabricated state or inert accepted configuration;
- every applicable `apply` cell changes real runtime behavior;
- policy remains in I2PControl wherever possible;
- startup/control-plane ownership remains distinct;
- HTTP/IRC/Streamr anonymity and resource bounds remain mandatory;
- server local targets remain confined as accepted;
- secrets and path-valued configuration remain redacted/confined;
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

The M095 machine-readable matrix remains authoritative.

Final closure requires:

- RouterInfo: all 43 rows available except proposal-permitted neutral semantics;
- AddressBook: all 13 SetConfig keys operational with explicit owners;
- TunnelManager: every canonical option/type cell is `apply` or evidenced `not_applicable`;
- zero applicable `planned_apply`, `blocked_primitive`, `unsupported`, or unknown cells;
- exactly 12 tunnel types and 7 actions;
- exactly 6 ClientServicesInfo selectors;
- compatibility aliases/extensions clearly separated from canonical completion.

Parser acceptance, persistence without runtime effect, or fail-before-allocation rejection is not final `apply` evidence.

## 9. Corrected dependency graph

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
  | transfers genuine primitive-dependent cells -----+
  v
M099 server/access/throttle independent slice
[READY — CURRENT HANDOFF]
  |
  | transfers genuine LeaseSet/session/unsafe cells -+
  v                                                  |
residual option blocker line <-----------------------+
[NO EXECUTABLE PLAN UNTIL A BOUNDED PRIMITIVE PATH EXISTS]
  |
  v
M104 live interoperability + final reclosure
[BLOCKED UNTIL ZERO RESIDUAL APPLICABLE CELLS]
```

M098 and M099 are serialized because they share matrix/option/filter artifacts. The server slice does not semantically depend on successful implementation of client behavior.

## 10. Milestone status and exit conditions

### M095 — full-support matrix and containment budget

Status: closed.

Exit evidence remains the exact 43 / 13 / 12×options / 6 inventory and path budgets.

### M096 — AddressBook SetConfig

Status: closed.

All 13 pinned keys now have bounded operational/metadata semantics. The sole non-I2PControl production amendment reused the existing AddressBook downloader seam rather than creating competing ownership.

### M097 — common tunnel session/key options

Status: closed as blocked.

Implemented safe subset:

- `TunnelLength`;
- `TunnelQuantity`;
- typed `EncType`.

Retained blockers include:

- `Shared`: no bounded compatible shared-session owner/handoff;
- `UseSSL`, `TunnelVariance`, `TunnelBackupQuantity`, `SigType`, `CustomOptions`: current Yosemite 0.7.0 SAM `SESSION CREATE` path does not emit required semantics;
- `NewDest`, `PersistentClientKey`: no accepted client destination/key lifecycle owner;
- `PrivKeyFile`: no confined validated import/store/handoff authority.

M097's stop condition remains valid. No speculative dependency fork is authorized.

### M098 — client proxy, management, and HTTP independent slice

Status: closed.

Before production code, M098 must reclassify every M098-owned cell using M097 closure evidence. It implements only exact behaviors owned by existing I2PControl client/proxy/filter/runtime surfaces and transfers genuine residual blockers out of M098.

Expected independent work:

- proxy/outproxy configuration;
- local proxy authentication/credential policy;
- direct-I2P vs explicit-outproxy routing;
- HTTP privacy/filter controls;
- generation-local client-management behavior only where exact semantics exist.

Exit: every cell still owned by M098 is `apply`; transferred blockers are explicit; no lower-layer/dependency expansion; M099 becomes current handoff. Met: applicable proxy/auth/privacy cells are applied and residual plugin/TLS-proxy/jump-list/management cells are named blockers.

### M099 — server access, throttle, and LeaseSet independent slice

Status: ready; current handoff.

M099 now consumes M098's final matrix ownership, implements server cells supported by existing accepted server/filter/admission runtime, and transfers genuine residual session/LeaseSet/unsafe-owner blockers.

Expected independent work:

- HTTP presentation/filter policy;
- peer access lists;
- confined filter files;
- connection ceilings;
- per-peer and aggregate rates;
- POST limits and periods;
- tunnel-local temporary denial.

LeaseSet encryption/auth/session-security cells remain blocked when exact supported Yosemite/SAM semantics do not exist.

Exit: every cell still owned by M099 is `apply`; no M093 security regression; residual blocker ledger exact.

### Residual option blocker line

Status: blocked; no registered implementation plan.

After M099, planning must inspect the residual matrix. A new implementation plan is registered only if current repository/dependency evidence shows a bounded solution inside accepted ownership.

A missing external/library primitive is a legitimate blocker. It is not permission to vendor or fork the dependency.

### M100 — transit 15s

Status: closed.

Request-independent I2PControl-owned sampler over the authoritative cumulative transit source.

### M101 — router news

Status: closed.

Bounded signed router-news acquisition/cache using the existing SU3/certificate seam. Live `.i2p` fetch evidence remains part of M104.

### M102 — network error owner

Status: closed.

Minimal neutral v4/v6 observation in existing core owners; Proposal 170 mapping remains in I2PControl.

### M103 — banned peers

Status: closed.

Explicit by-design-empty router-wide banned-peer source after exhaustive proof that Emissary has no such ban owner. No ban algorithm added.

### M104 — full live interoperability/reclosure

Status: blocked.

M104 cannot start until revised M098/M099 close and every residual applicable TunnelManager cell is resolved by a separately registered closure.

Exit: full integrated matrix, live data-plane evidence for all 12 tunnel families, AddressBook/RouterInfo/ClientServicesInfo validation, security/containment reclosure, and revision-pinned support statement.

## 11. Security and anonymity requirements

Corrective option work must preserve:

- trusted Yosemite-derived peer identity;
- bounded transactional server admission;
- literal-loopback server targets;
- HTTP framing/spoof/fingerprint/Expect/POST protections;
- IRC registration/DCC/CTCP/lifetime protections;
- bounded Streamr subscriber/payload/fanout state;
- secret-safe persistence/get/logging;
- explicit I2P outproxy requirement for clearnet proxy traffic;
- no local DNS fallback for direct I2P destinations;
- no silent LeaseSet security downgrade.

Any high/medium regression creates a new corrective plan rather than being absorbed into M104.

## 12. Failure, lifecycle, and contention requirements

- validation before allocation/publication where technically possible;
- failed edits preserve prior durable/runtime generations;
- per-name tunnel lifecycle remains serialized;
- timers/workers/tasks remain generation-local and cancellable;
- no lock crosses network I/O, sleeps, joins, or cancellation waits;
- bounded cleanup on stop/restart;
- blocked options continue to fail before allocation;
- no partial state presented as successful configuration.

## 13. Verification policy

Use focused unit/integration/static guards plus the existing broad feature-gated suite. Do not build a CI farm for this phase.

Expected baseline commands for implementation passes:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

The historical `m063_feature_reachability` test target is absent in the current checkout; preserve that limitation rather than inventing unrelated replacement scope.

The repository-wide nightly/stable rustfmt mismatch is a tooling issue. Do not rewrite audited core files solely for formatter churn.

## 14. Risks and deferred work

### Residual Yosemite/SAM capability gaps

Risk: final full support may remain blocked if the supported dependency cannot express exact pinned session semantics.

Response: fail closed and keep the blocker explicit. A future contained primitive plan requires current evidence; no automatic vendor/fork path.

### Client identity/shared-session ownership

Risk: implementing shared sessions or persistent/new client identities can create cross-tunnel ownership, identity rotation, secret persistence, and restart hazards.

Response: no opportunistic implementation inside M098. Any bounded owner requires a separate registered plan if/when the residual line is reopened.

### Server option security

Risk: access/rate/presentation options could weaken M093 anonymity or create unbounded peer state.

Response: compose with existing trusted identity/admission/filter owners; no parallel security stack.

### Live-network evidence

Risk: local verification may not have a functional I2P network/reference router.

Response: keep M104 blocked rather than presenting process-local success as network interoperability.

## 15. Full-support exit condition

This roadmap closes only when M104 can truthfully record:

> Emissary fully supports I2P Proposal 170 against the pinned 2026-05-20 revision.

That claim requires zero applicable blocked/unsupported/planned TunnelManager cells and focused live interoperability. It does not mean general I2PControl parity or upstream acceptance.

## 16. Internal-only boundary

All work and writes remain internal to `eggstack/emissary`. External specifications, Yosemite, Java I2P, i2pd, issues, commits, and pull requests are read-only evidence only.

No upstream issue/PR/review/submission/merge/adoption request, contribution preparation, branch/tag push, release, or maintainer contact is authorized by this roadmap.
