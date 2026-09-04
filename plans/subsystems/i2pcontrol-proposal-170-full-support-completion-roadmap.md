# I2PControl Proposal 170 Full-Support Completion Roadmap

Status: **active / partial; M134/M135/M136/M137 closed as complete, lifecycle line complete**

Current handoff:

- none from the lifecycle line (M134 closed as complete, six promotions, `325/47/468`).

Closed correctives:

- M135 closed as complete (`plans/closure/i2pcontrol-proposal-170/135-closure.md`), zero promotions;
- M136 closed as complete (`plans/closure/i2pcontrol-proposal-170/136-closure.md`), 21 `Reduce*` promotions;
- M137 closed as complete (`plans/closure/i2pcontrol-proposal-170/137-closure.md`), 14 `Close`/`CloseTime` promotions.

Focused lifecycle roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md`.

Corrective successors already planned but unregistered:

- none from the lifecycle line (complete).

Historical lifecycle attempts:

- M132 closed as blocked;
- M133 closed as blocked;
- M134 is closed as complete (`plans/closure/i2pcontrol-proposal-170/134-closure.md`, six `NewDest` promotions).

Current authorities:

- runtime/security qualification: M130 closure `plans/closure/i2pcontrol-proposal-170/130-closure.md`;
- residual applicability/primitive authority: M131 closure `plans/closure/i2pcontrol-proposal-170/131-closure.md` and `131-residual-primitive-map.toml`;
- current matrix: `325 apply / 47 blocked_primitive / 468 not_applicable`;
- M136 closure: `plans/closure/i2pcontrol-proposal-170/136-closure.md`;
- M137 closure: `plans/closure/i2pcontrol-proposal-170/137-closure.md`.

Pinned external authority:

- I2P Proposal 170 revision `2026-05-20`, status Open;
- read-only Java I2P lifecycle reference snapshot `i2p/i2p.i2p@2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243` for M135-M137.

All external specification/reference activity is read-only. Repository writes remain internal to `eggstack/emissary`; Yosemite writes require separate ADR-0005/Yosemite planning authority and are not authorized by M134-M137.

## 1. Purpose

Move the internal fork from truthful partial Proposal-170 support toward exact support while keeping Proposal-specific business/admin/application policy under `emissary-cli/src/i2pcontrol/**` wherever possible and preserving audited core/router boundaries.

Full support means real externally observable behavior. Parser acceptance, persisted values, dormant serializer fields, fabricated defaults or approximate semantics do not count.

This is not a general base-I2PControl parity program, router redesign, frontend project or upstream contribution program.

## 2. Canonical authority

Read in order:

1. `plans/000-long-term-specification.md`;
2. `plans/001-terminology-and-domain-model.md`;
3. `plans/002-long-term-roadmap.md`;
4. `plans/003-planning-process.md`;
5. ADR-0001 through ADR-0005;
6. M061/M062 containment;
7. M093 tunnel security;
8. M095 full-support matrix;
9. M105 residual audit;
10. M110 completion ledger;
11. M130/M131 closures;
12. this roadmap, focused subsystem roadmap, registry and the specific registered plan.

Historical closures remain immutable evidence. Corrective plans reference them rather than rewriting them.

## 3. Current support state

Qualified implemented subset includes:

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable;
- AddressBook CRUD, subscriptions and all 13 SetConfig keys;
- all 12 canonical TunnelManager data planes and seven actions for the claimed subset;
- all six ClientServicesInfo selectors;
- finite token lifetime, bounded JSON-RPC batches and fail-closed non-loopback management TLS;
- shared-session/destination ownership from M110/M116;
- neutral tunnel variance/backup behavior from M118/M119;
- previously qualified application/security behavior for supported tunnel fields.

M130 remains current implemented-subset runtime/security authority.

M131 corrected eight false applicability blockers with zero runtime promotion. Current Proposal matrix remains:

- `284 apply`;
- `88 blocked_primitive`;
- `468 not_applicable`.

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

M118/M119 are historical lower-layer precedent. M135 is the current explicit neutral-core exception.

Yosemite remains the sole accepted SAM client for I2PControl. Exact Y005 stays isolated behind the optional `yosemite-i2pcontrol` alias. No global patch, vendoring, path override, floating fork or parallel raw SAM is authorized.

## 5. Cross-cutting invariants

All remaining work preserves:

- exact pinned names/types/actions/presence semantics;
- no fabricated or accept-inert support;
- every `apply` cell changes real runtime behavior;
- unsupported supplied values fail before allocation/effect;
- no direct-I2P-to-clearnet DNS fallback;
- trusted peer identity and Streamr producer isolation;
- local-target confinement and existing tunnel security boundaries;
- bounded admission/tasks/timers/state and generation-local cancellation;
- transactional edit/start/restart and last-known-good preservation;
- no lock across unrelated network/filesystem I/O, sleeps, joins or timer waits;
- secret/key/path redaction and confinement;
- no LeaseSet security downgrade;
- feature-disabled/runtime-disabled isolation;
- no unrelated base method parity;
- no frontend coupling;
- external interaction read-only/internal-only.

## 6. Residual primitive state

The 88 blocked cells remain partitioned under M131 authority. Major clusters include:

- session lifecycle/reduction/close/resume;
- 10 `SigType` destination-signing cells;
- 15 encrypted/authenticated LeaseSet cells;
- 7 streaming `Profile` cells;
- 4 presentation `UseSSL` cells;
- 4 `UseOutproxyPlugin` cells;
- HTTP-only `SSLProxies` and `JumpList` residual cells;
- 2 `UniqueLocalAddressPerClient` cells;
- 2 `MultiHoming` / `shouldBundleReplyInfo` cells;
- retained Streamr residuals such as `ConnectDelay`.

M131's machine-readable map remains the authority for exact cells and path-budget identities.

## 7. Active corrective lifecycle line

The M132/M133 attempts closed blocked with zero production/runtime promotion. Their main value was exposing a decomposition problem: live pool reconfiguration, LeaseSet convergence, session activity/timers and Proposal translation were coupled into one milestone.

Direct read-only Java source subsequently resolved the key reference unknowns:

- runtime quantity change is a client-pool settings reconfiguration;
- excess live tunnels are not synchronously purged; the lower desired quantity stops future replacement/build above target while existing tunnels expire normally;
- LeaseSet wanted count follows current inbound quantity;
- Streamr uses a normal generic I2PSession and therefore the same session-level reduce/close idle monitor;
- idle defaults/order/restore semantics are explicit in `SessionIdleTimer` and `I2PSessionImpl`.

The corrective chain is:

```text
M131 residual primitive re-freeze                         [CLOSED AS BLOCKED — 284/88/468]
  |
  +--> M132 combined reduction attempt                    [CLOSED AS BLOCKED]
  |      x
  +--> M133 combined close attempt                        [CLOSED AS BLOCKED]
  |
  v
M135 neutral live quantity + LeaseSet desired count       [CLOSED AS COMPLETE]
  |
  v
M136 M132 corrective: session activity + Reduce*          [CLOSED AS COMPLETE — 305/67/468]
  |
  v
M137 M133 corrective: Close* + reasoned termination       [CLOSED AS COMPLETE — 319/53/468]
  |
  v
M134 NewDest on proven idle resume                        [CLOSED AS COMPLETE — 325/47/468]
```

No active implementation handoff from the lifecycle line (complete).

### M135

M135 closed as complete (`plans/closure/i2pcontrol-proposal-170/135-closure.md`) with the matrix unchanged at `284/88/468`.

It established:

- a generic current desired inbound/outbound quantity distinct from immutable base config;
- reference-compatible no-immediate-purge convergence;
- bounded destination-scoped control;
- dynamic LeaseSet desired inbound count;
- destination coordination of pool/LeaseSet target updates.

No SAM or I2PControl production source is authorized under M135.

### M136

M136 closed as complete (`plans/closure/i2pcontrol-proposal-170/136-closure.md`, 21 promotions, `305/67/468`). It corrected M132 with one generation-local SAM session activity/timer owner and Proposal `Reduce`, `ReduceTime`, `ReduceCount` translation through standard I2CP options.

### M137

M137 closed as complete (`plans/closure/i2pcontrol-proposal-170/137-closure.md`, 14 promotions, `319/53/468`). It extended the same owner with standard close-on-idle semantics, canonical session teardown and a neutral authoritative termination cause, then mapped Proposal `Close`/`CloseTime`.

It did not implement `NewDest`.

### M134 NewDest (closed)

M134 closed as complete (`plans/closure/i2pcontrol-proposal-170/134-closure.md`, six promotions, `325/47/468`). Historical M134 was rebased on the proven M137 §12 contract (no M138 required).

## 8. Other residual clusters

M134-M137 do not authorize work on:

- presentation TLS;
- HTTP SSL-outproxy/jump behavior;
- outproxy provider/plugin integration;
- streaming profile/window configuration;
- per-client local source address;
- `shouldBundleReplyInfo` sender LeaseSet bundling;
- destination signing-type generation;
- encrypted/authenticated LeaseSets.

Those remain unregistered until selected under the planning ceremony.

## 9. Historical authority

| Milestone | Current role |
|---|---|
| M061/M062 | containment/dependency authority |
| M093 | tunnel application/security boundary |
| M095 | machine-readable support matrix |
| M105 | residual-option audit |
| M110/M116 | shared-session/destination ownership |
| M117 | exact optional Yosemite dependency seam |
| M118/M119 | neutral variance/backup behavior |
| M121 | SigType and Close/CloseTime/NewDest truthfulness demotion |
| M122/M124 | exact Yosemite Y004/Y005 transport adoption |
| M123 | cancellation/commit atomicity |
| M125 | AllowInternalSSL applicability correction |
| M127-M129 | token/batch/TLS shared-control-plane corrections |
| M130 | current implemented-subset runtime/security qualification |
| M131 | current residual applicability/primitive authority |
| M132 | closed blocked; failed combined reduction vertical slice |
| M133 | closed blocked; failed dependent close vertical slice |

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

Serializer reachability alone is not capability readiness.

## 11. Verification policy

Implementation milestones run focused suites plus relevant core/CLI checks, M061/M062 containment, M095/M105 matrix tests, live runtime where applicable, clippy and `git diff --check`. `cargo fmt --all -- --check` is attempted and pre-existing stable/nightly drift is recorded without unrelated normalization.

No new hosted CI/fuzz/release orchestration is required by this roadmap.

## 12. Registration discipline

Per `plans/003-planning-process.md`:

- **M136 is dependency-ready and becomes the sole active handoff on its own
  registration step (status flip + gate citation)**;
- M137 remains unregistered until M136 closure explicitly declares dependency readiness;
- M137 remains unregistered until M136 closure explicitly declares dependency readiness;
- M134 is closed as complete; no M138 exists;
- all other residual clusters remain unregistered;
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

Until then official status remains **partial Proposal 170 support**.