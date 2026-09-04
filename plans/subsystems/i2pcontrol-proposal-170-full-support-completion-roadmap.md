# I2PControl Proposal 170 Full-Support Completion Roadmap

Status: **active / partial; M132/M133 closed as blocked; no active handoff**

Current registered handoff: none — M132/M133 are closed as blocked
(`plans/closure/i2pcontrol-proposal-170/132-closure.md`,
`plans/closure/i2pcontrol-proposal-170/133-closure.md`).

Prior handoffs:

- `plans/implementation/i2pcontrol-proposal-170/132-neutral-sam-idle-reduction-and-proposal-reduce-completion.md` — **closed as blocked**;
- `plans/implementation/i2pcontrol-proposal-170/133-neutral-sam-idle-close-and-reasoned-termination.md` — **closed as blocked**.

Active focused roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md`.

Current authorities:

- runtime/security qualification: M130 closure `plans/closure/i2pcontrol-proposal-170/130-closure.md`;
- residual applicability/primitive authority: M131 closure `plans/closure/i2pcontrol-proposal-170/131-closure.md` and `131-residual-primitive-map.toml`;
- current matrix: `284 apply / 88 blocked_primitive / 468 not_applicable`;
- M131 closure head / M132 planning baseline: `3a829d7d3d6314ecf09e42dbf0339506f0917c96`.

Pinned external authority:

- I2P Proposal 170, revision `2026-05-20`, status Open.

All external specification/reference activity is read-only. Repository writes remain internal to `eggstack/emissary`; Yosemite writes require separate ADR-0005/Yosemite planning authority.

## 1. Purpose

Move the internal fork from truthful partial Proposal-170 support toward exact support against the pinned revision while keeping Proposal-specific business/admin/application policy under `emissary-cli/src/i2pcontrol/**` wherever possible and preserving audited core/router boundaries.

Full support means real externally observable behavior. Parser acceptance, storage, serializer reachability, dormant fields, fabricated defaults or approximate semantics do not count.

This is not a general base-I2PControl parity program, router redesign, frontend project or upstream contribution program.

## 2. Canonical/internal authority

Read in order:

1. `plans/000-long-term-specification.md`;
2. `plans/001-terminology-and-domain-model.md`;
3. `plans/002-long-term-roadmap.md`;
4. `plans/003-planning-process.md`;
5. ADR-0001 through ADR-0005;
6. M061/M062 containment;
7. M093 tunnel-security authority;
8. M095 full-support matrix;
9. M105 residual audit;
10. M110 completion ledger;
11. current closure authorities M130/M131;
12. this roadmap, focused subsystem roadmaps, registry and registered implementation plan.

Historical closures remain immutable evidence. Later correctives/audits supersede only explicitly affected claims.

## 3. Current support state

Qualified implemented subset includes:

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable;
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and deterministic effective lookup;
- all 12 canonical TunnelManager data planes and all seven canonical actions for the claimed subset;
- all six ClientServicesInfo selectors;
- API-1 auth/version/token behavior required by the extension surface;
- HTTPS-only management serving, bounded JSON-RPC batch semantics, finite token lifetime and fail-closed non-loopback TLS;
- shared-session/destination ownership from M110/M116;
- neutral tunnel variance/backup behavior from M118;
- already-closed HTTP/IRC/SOCKS/Streamr application/security behavior.

M130 remains the current implemented-subset runtime/security qualification authority.

M131 corrected the residual ledger without runtime promotion:

- 8 cells moved from blocked to not applicable;
- zero cells moved to apply;
- current authority is `284 / 88 / 468`.

Full Proposal 170 support is **not** claimed.

## 4. Ownership and containment

Proposal policy belongs under `emissary-cli/src/i2pcontrol/**`.

A production change outside that boundary is permitted only when:

1. behavior belongs to an existing canonical lower-layer owner;
2. no truthful I2PControl-local implementation exists;
3. exact paths are named before implementation;
4. the seam is neutral, not Proposal-shaped;
5. unrelated router behavior is unchanged;
6. M061/M062 exact-path evidence is amended;
7. a registered plan authorizes the change.

M118 is historical precedent. M132 is the current explicit exception for generic SAM session activity and live tunnel-pool quantity control.

Yosemite remains the sole accepted SAM client for I2PControl. Exact Y005 stays isolated behind the optional `yosemite-i2pcontrol` alias. No global patch, vendoring, path override, floating fork or parallel raw SAM is authorized.

## 5. Cross-cutting invariants

All remaining work preserves:

- exact pinned names/types/actions/presence semantics;
- no fabricated/accept-inert support;
- every `apply` cell changes real runtime behavior;
- unsupported supplied values fail before allocation/effect;
- no direct-I2P-to-clearnet DNS fallback;
- explicit accepted I2P outproxy boundary for clearnet proxying;
- trusted peer identity and Streamr producer isolation;
- literal-loopback/local-target confinement unless a separately accepted neutral primitive proves equivalent safety;
- bounded admission/tasks/timers/state and generation-local cancellation;
- transactional edit/start/restart and last-known-good preservation;
- no lock across unrelated network/filesystem I/O, sleeps, joins or timer waits;
- secret/key/path redaction/confinement;
- no LeaseSet security downgrade;
- feature-disabled/runtime-disabled isolation;
- no unrelated base method parity;
- no frontend coupling;
- external interaction read-only/internal-only.

## 6. Residual primitive state after M131

The 88 blocked cells now have explicit primitive owners. Major clusters are:

- session lifecycle/reduction/close/resume;
- 10 `SigType` destination-signing cells;
- 15 encrypted/authenticated LeaseSet cells;
- 7 streaming `Profile` cells;
- 4 presentation `UseSSL` cells;
- 4 `UseOutproxyPlugin` cells;
- HTTP-only `SSLProxies` and `JumpList` residual cells;
- 2 `UniqueLocalAddressPerClient` cells;
- 2 `MultiHoming` / `shouldBundleReplyInfo` cells;
- retained Streamr lifecycle ambiguities such as `ConnectDelay`.

M131's machine-readable map is the authority for exact cells and path-budget IDs.

## 7. Active session-lifecycle line

M131 identified the session-lifecycle cluster as the highest-leverage neutral prerequisite. Current code review after M131 resolved enough owner/interface detail to make M132 dependency-ready:

- `SamSession` owns actual I2CP payload activity;
- `Destination` owns the destination's pool handle;
- `TunnelPool` owns active/standby capacity;
- the I2PControl session adapter owns Proposal-to-Yosemite translation;
- Yosemite Y005 can carry standard I2CP options without modification.

The line is:

```text
M131 residual primitive re-freeze                 [CLOSED AS BLOCKED]
  |
  v
M132 idle reduction + live pool target            [CLOSED AS BLOCKED]
  |
  x
M133 idle close + reasoned termination            [CLOSED AS BLOCKED]
  |
  x
M134 NewDest on proven idle resume                [DEFERRED / UNREGISTERED]
```

No handoff is executable now. M132/M133 closure unblocks no successor.

### M132

Plan: `132-neutral-sam-idle-reduction-and-proposal-reduce-completion.md`.

M132 implements a generic generation-local SAM activity clock, bounded live active inbound/outbound pool target and restore path, then maps Proposal `Reduce`, `ReduceCount`, `ReduceTime` through Yosemite's existing session-option boundary.

It may promote up to 21 mechanically present `Reduce*` cells only if reference evidence proves Streamr/datagram applicability. Otherwise the three Streamr cells remain blocked. End-to-end behavior, not count, controls closure.

M132 authorizes no Cargo/Yosemite dependency changes and no unrelated core path.

### M133

Plan: `133-neutral-sam-idle-close-and-reasoned-termination.md` — **closed as blocked** (`plans/closure/i2pcontrol-proposal-170/133-closure.md`).

Hard-depended on M132 closure. It would have reused the M132 activity/timer state for `Close`/`CloseTime`, performed bounded idle session teardown, and provided a neutral authoritative in-process idle-close reason without a SAM wire extension. Zero `Close`/`CloseTime` cells promoted: M132 provided no owner to extend.

### M134

Plan exists but is unregistered: `134-newdest-on-proven-idle-resume.md`.

Hard-depends on M133 (not satisfied — M133 closed as blocked without an authoritative reason). It keeps `NewDest` entirely I2PControl-owned, stages/commits a fresh destination exactly once on successful resume after a proven idle close, and never rotates on manual stop/start, restart, process restart, network failure or failed/cancelled resume. Streamr NewDest remains not applicable under M131.

## 8. Other residual clusters

The active session-lifecycle line does not authorize work on:

- presentation TLS;
- HTTP SSL-outproxy/address-helper behavior;
- outproxy provider/plugin integration;
- streaming profile/window configuration;
- per-client local source address;
- `shouldBundleReplyInfo` sender LeaseSet bundling;
- destination signing-type generation;
- encrypted/authenticated LeaseSets.

Those remain unregistered until their own dependency-ready plans are selected after the active line or a separately justified priority change.

## 9. Historical authority

| Milestone | Current role |
|---|---|
| M061/M062 | containment/dependency authority |
| M093 | tunnel application/security boundary |
| M095 | machine-readable support matrix |
| M105 | residual-option audit |
| M110/M116 | shared-session/destination ownership and corrective authority |
| M111 | historical session-wire applied subset / UseSSL blocker |
| M112 | historical client proxy/lifecycle closure |
| M113 | historical server residual closure |
| M117 | exact optional Yosemite dependency seam |
| M118/M119 | neutral variance/backup behavior and corrective |
| M121 | SigType and Close/CloseTime/NewDest truthfulness demotion |
| M122/M124 | exact Yosemite Y004/Y005 transport adoption |
| M123 | cancellation/commit atomicity |
| M125 | AllowInternalSSL applicability correction and capability audit |
| M127-M129 | token/batch/TLS shared-control-plane corrections |
| M130 | current implemented-subset runtime/security qualification |
| M131 | current residual applicability/primitive authority |

## 10. Successor readiness requirements

No future primitive is dependency-ready until it defines:

- exact externally observable effect;
- canonical owner and path budget;
- validation-before-effect boundary;
- allocation/publication point;
- cancellation/generation/restart owner;
- bounded state/queue/timer semantics;
- lock/contention behavior;
- rollback/last-known-good behavior;
- security/secret implications;
- deterministic focused tests;
- reference/live interoperability requirements where applicable.

An interface that merely serializes a value is not capability readiness.

## 11. Verification policy

Implementation milestones run focused suites plus relevant core/CLI checks, M061/M062 containment, M095/M105 matrix tests, live runtime where applicable, clippy and `git diff --check`. `cargo fmt --all -- --check` is attempted and pre-existing stable/nightly drift is recorded without unrelated normalization.

No new hosted CI/fuzz/release orchestration is required by this roadmap.

## 12. Registration discipline

Per `plans/003-planning-process.md`:

- M132/M133 are closed as blocked; M134 stays unregistered until a future reduction primitive and a future close primitive explicitly prove the authoritative idle-close/reopen semantics;
- future residual clusters remain unregistered;
- material deviations require plan amendment before production code;
- closure evidence decides support and matrix promotion.

## 13. Final completion rule

Full Proposal 170 completion requires:

- zero applicable residual primitive gaps against the pinned revision;
- every applied cell backed by real runtime behavior;
- no high/medium Proposal-scoped correctness/security defect;
- bounded reference/live interoperability evidence;
- minimal explained non-I2PControl production seams;
- final whole-surface requalification after the last capability closes.

Until then the official status remains **partial Proposal 170 support**.