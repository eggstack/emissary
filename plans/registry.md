# Emissary Active Planning Registry

This file is the compact control surface for active planning.

Canonical direction:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`

Accepted Proposal-170 architecture/security authority:

- ADR-0001 through ADR-0005;
- M061/M062 containment;
- M093 tunnel security.

Pinned Proposal 170 revision: `2026-05-20` (Open).

Authorized internal repositories:

- `eggstack/emissary`;
- `eggstack/yosemite` only under ADR-0005 and Yosemite's own registered plans.

All upstream/third-party repositories and maintainer channels remain read-only.

## Active roadmaps

| Subsystem | Status | Roadmap | Current handoff |
|---|---|---|---|
| Proposal 170 full-support completion | **active / partial** | `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` | **M132 closed as blocked; no active handoff** |
| Proposal 170 session-lifecycle completion | **active / blocked** | `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md` | **M132 closed as blocked**; M133-M134 deferred |
| Post-M114 shared-control-plane corrective line | closed | `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md` | M130 closed as current runtime/security authority |
| I2PControl containment | accepted authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | M061/M062 regression authority |

## M132 closure — no active handoff

Plan:

- `plans/implementation/i2pcontrol-proposal-170/132-neutral-sam-idle-reduction-and-proposal-reduce-completion.md`

Status: **closed as blocked**.

Baseline:

- M131 closure head `3a829d7d3d6314ecf09e42dbf0339506f0917c96`;
- M131 matrix `284 apply / 88 blocked_primitive / 468 not_applicable`.

Closure:

- `plans/closure/i2pcontrol-proposal-170/132-closure.md`.

M132 was the Proposal-170 implementation handoff for the smallest neutral
lower-layer session/pool primitive required by Proposal `Reduce`,
`ReduceCount`, and `ReduceTime`. It closed as blocked with zero promotions:
reference items 9–11 (excess-tunnel behavior, LeaseSet convergence, Streamr
ownership) could not be resolved without guessing, the live pool target plus
truthful LeaseSet synchronization requires a broad redesign, Yosemite typed
reduce fields are dormant, and Streamr applicability remains ambiguous. The
matrix remains `284 apply / 88 blocked_primitive / 468 not_applicable`.

No Proposal-170 implementation handoff is active or registered after M132
closure. M133/M134 remain deferred/unregistered.

## Deferred session-lifecycle successors

### M133 — idle close and reasoned termination

Plan:

- `plans/implementation/i2pcontrol-proposal-170/133-neutral-sam-idle-close-and-reasoned-termination.md`

Status: **deferred / unregistered**; hard dependency on M132 closure (not satisfied — M132 closed as blocked without a stable activity/timer owner).

M133 would reuse the M132 activity/timer owner for `Close`/`CloseTime` and add only a neutral authoritative in-process idle-close termination reason. It must not implement `NewDest` or invent a SAM wire extension. Registration requires a future proven reduction primitive first.

### M134 — NewDest on proven idle resume

Plan:

- `plans/implementation/i2pcontrol-proposal-170/134-newdest-on-proven-idle-resume.md`

Status: **deferred / unregistered**; hard dependency on M133 closure.

M134 keeps destination/key rotation entirely I2PControl-owned. It may consume one authoritative M133 idle-close fact and rotate exactly once on a successful qualifying resume. Manual stop/start, restart, process restart, network/SAM failure and failed/cancelled resume must not rotate. `NewDest:streamrclient` remains not applicable under M131 authority.

## Current production/support state

Current implemented-subset runtime/security qualification authority remains M130:

- closure `plans/closure/i2pcontrol-proposal-170/130-closure.md`;
- implementation head `fe1a981`;
- closure head `a68094e128d2b92f0fd5b350e38512ef6b65cb6b`.

M131 is the current residual applicability/primitive authority:

- closure `plans/closure/i2pcontrol-proposal-170/131-closure.md`;
- primitive map `plans/implementation/i2pcontrol-proposal-170/131-residual-primitive-map.toml`;
- closure head `3a829d7d3d6314ecf09e42dbf0339506f0917c96`.

Current supported surface according to closure evidence:

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable;
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and cross-book precedence operational;
- all 12 canonical TunnelManager data planes and seven actions exist for the claimed subset;
- all six ClientServicesInfo selectors operational;
- finite API token lifetime, bounded JSON-RPC batches and fail-closed non-loopback management TLS qualified by M127-M130.

Current M095 matrix:

- `284 apply`;
- `88 blocked_primitive`;
- `468 not_applicable`.

Full Proposal 170 status remains **partial**.

## Dependency graph

```text
M130 integrated requalification             [CLOSED — CURRENT RUNTIME AUTHORITY]
  |
  v
M131 residual primitive re-freeze           [CLOSED AS BLOCKED — 284/88/468]
  |
  v
M132 idle reduction + live pool target      [CLOSED AS BLOCKED — 284/88/468]
  |
  x
M133 idle close + reasoned termination      [DEFERRED / UNREGISTERED — M132 did not provide the stable activity/timer owner]
  |
  x
M134 NewDest on proven idle resume          [DEFERRED / UNREGISTERED — hard-depends on M133]
```

M114 remains historically closed as blocked. The M132-M134 line resolves only the session-lifecycle cluster and does not authorize unrelated residual clusters. M132 closure unblocks no successor.

## Residual clusters outside the active line

Remain unregistered under M131 authority:

- presentation `UseSSL`;
- `SigType` destination signing;
- `UseOutproxyPlugin`;
- HTTP `SSLProxies` / `JumpList`;
- streaming `Profile`;
- Streamr `ConnectDelay` if still ambiguous;
- `UniqueLocalAddressPerClient`;
- `MultiHoming` / `shouldBundleReplyInfo`;
- encrypted/authenticated LeaseSets.

No successor from those clusters may be smuggled into M132-M134.

## Canonical containment rules

1. Proposal policy stays under `emissary-cli/src/i2pcontrol/**` wherever possible.
2. M132 closed with no core changes; any future reduction primitive is permitted only in its exact neutral SAM/destination/tunnel-pool path budget and must be reflected in M062 before implementation closure.
3. No Proposal-shaped `emissary-core` API is accepted merely to improve matrix counts.
4. Yosemite remains the sole accepted SAM implementation; no parallel raw SAM stack.
5. Exact Y005 remains isolated behind the optional `yosemite-i2pcontrol` alias.
6. No global patch/path/vendor/floating Yosemite dependency.
7. No direct-clearnet fallback, loopback-confinement weakening, TLS verification bypass or LeaseSet security downgrade.
8. No secrets in RPC/log/Debug/planning evidence.
9. No frontend/startup/config rewrite unless separately authorized.
10. External/upstream activity remains read-only.

## Registration rules

1. **No Proposal-170 implementation plan is active/registered after M132 closure.**
2. M133 is registered only after a future reduction primitive explicitly proves the M132 dependency-ready interface (stable activity/timer owner, generation-local lifecycle contract); M132 closure explicitly did not prove it.
3. M134 is registered only after M133 closure explicitly proves an authoritative idle-close reason/reopen contract.
4. Material path/architecture deviations require plan amendment before code.
5. Closure evidence, not implementation assertions, determines support/matrix promotion.
6. Active documentation must retain partial-support wording until all applicable residuals are resolved and requalified.

## Recently closed / current authority

| Milestone | Disposition |
|---|---|
| M121 | closed; semantic truthfulness demotion of `SigType` and `Close`/`CloseTime`/`NewDest` |
| M123 | closed; commit-phase cancellation/lifecycle atomicity |
| M125 | closed; two `AllowInternalSSL` cells corrected to not applicable |
| M127 | closed; finite token lifetime |
| M128 | closed; bounded JSON-RPC batch conformance |
| M129 | closed; non-loopback managed-TLS fail-closed |
| M130 | closed; current implemented-subset runtime/security qualification |
| M131 | closed as blocked; residual applicability/primitive re-freeze; 8 applicability corrections; matrix 284/88/468 |
| M132 | **closed as blocked**; idle reduction + live pool target; zero promotions; M133/M134 remain deferred |

Historical closure files remain unchanged.