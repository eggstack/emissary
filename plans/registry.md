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
| Proposal 170 full-support completion | **active / partial** | `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` | **lifecycle complete** (M134 closed as complete; 47 non-lifecycle residuals unregistered under M131) |
| Proposal 170 session-lifecycle completion | **closed as complete** | `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md` | M134 closed as complete (lifecycle line complete) |
| Post-M114 shared-control-plane corrective line | closed | `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md` | M130 current runtime/security authority |
| I2PControl containment | accepted authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | M061/M062 regression authority |

## Active Proposal-170 implementation handoff

No active implementation handoff. M134 is closed as complete (see below); the
session-lifecycle line is complete and the remaining 47 non-lifecycle residuals
stay unregistered under M131 authority. No successor is authorized by M134.

### M134 — NewDest on proven idle resume (closed)

Plan/closure:

- `134-newdest-on-proven-idle-resume.md`;
- `plans/closure/i2pcontrol-proposal-170/134-closure.md`.

Status: **closed as complete**, six promotions, matrix `325/47/468`. M134
rebases historical NewDest design material on the proven M137 §12
reason/reopen contract and implements exact Proposal `NewDest` for the six
non-Streamr TCP families with one-shot tracker-proven rotation, staged secret
transaction, one shared successor and manual/restart/failure preservation.

### M137 — M133 corrective: SAM idle close and reasoned termination (closed)

Plan/closure:

- `137-m133-corrective-sam-idle-close-and-reasoned-termination.md`;
- `plans/closure/i2pcontrol-proposal-170/137-closure.md`.

Status: **closed as complete**, 14 promotions, matrix `319/53/468`. M137
extends the same M136 activity/timer owner with standard
`i2cp.closeOnIdle`/`i2cp.closeIdleTime`, exact close-before-reduce
ordering, canonical session teardown and a neutral authoritative
generation-local termination cause, and maps Proposal `Close`/`CloseTime`
through Yosemite's validated generic session-option path.

## Closed primitives

### M135

Plan/closure:

- `135-neutral-live-tunnel-quantity-and-leaseset-reconfiguration-primitive.md`;
- `plans/closure/i2pcontrol-proposal-170/135-closure.md`.

Status: **closed as complete**, zero promotions, matrix `284/88/468`
unchanged at its closure (now superseded by M136 `305/67/468`). M135 proved
the stable destination-scoped target update/restore,
reference-compatible convergence, dynamic LeaseSet desired count, bounded
generation-local control, clean containment, and no unresolved high/medium
defect required by the M136 gate.

### M136

Plan/closure:

- `136-m132-corrective-sam-idle-reduction-and-proposal-reduce-completion.md`;
- `plans/closure/i2pcontrol-proposal-170/136-closure.md`.

Status: **closed as complete**, 21 promotions, matrix `305/67/468`. M136
proved one canonical SAM idle activity/timer owner with standard
`i2cp.reduce*` consumption through M135, Proposal `Reduce*` validation and
Yosemite generic mapping for all seven client families including Streamr,
shared-session exact identity, deterministic shutdown/isolation, and no
unresolved high/medium defect required by the M137 gate.

### M137

Plan/closure:

- `137-m133-corrective-sam-idle-close-and-reasoned-termination.md`;
- `plans/closure/i2pcontrol-proposal-170/137-closure.md`.

Status: **closed as complete**, 14 promotions, matrix `319/53/468` (now
superseded by M134 `325/47/468`). M137
extends the same owner with standard `i2cp.close*` close-before-reduce
ordering, canonical teardown, neutral `IdlePolicy`/`Requested`/`Failure`/
`Unknown` reason, and Proposal `Close`/`CloseTime` validation plus Yosemite
generic mapping for all seven client families including Streamr.

### M134

Plan/closure:

- `134-newdest-on-proven-idle-resume.md`;
- `plans/closure/i2pcontrol-proposal-170/134-closure.md`.

Status: **closed as complete**, six promotions, matrix `325/47/468`. M134
closes the session-lifecycle line: `NewDest` rotates exactly once on a proven
`IdlePolicy` resume for the six TCP families with Streamr staying N/A.

## Deferred corrective successors

No deferred lifecycle successor remains. M138 was not required: historical
M134 assumptions matched the M137 contract after rebase.

## Closed failed predecessor line

### M132

Plan/closure:

- `132-neutral-sam-idle-reduction-and-proposal-reduce-completion.md`;
- `plans/closure/i2pcontrol-proposal-170/132-closure.md`.

Status: **closed as blocked**, zero promotions, no production implementation.

M132 correctly stopped because its combined vertical slice could not freeze excess-tunnel, LeaseSet and Streamr reference behavior in that execution and its path budget did not provide an independently proven lower-layer primitive.

### M133

Plan/closure:

- `133-neutral-sam-idle-close-and-reasoned-termination.md`;
- `plans/closure/i2pcontrol-proposal-170/133-closure.md`.

Status: **closed as blocked**, zero promotions, no production implementation.

M133 was blocked because M132 never produced the canonical activity/timer owner it hard-depended on.

Historical closures remain unchanged; M135-M137 are explicit correctives, not revisions of closure evidence.

## Current production/support state

Current implemented-subset runtime/security authority remains M130:

- closure `plans/closure/i2pcontrol-proposal-170/130-closure.md`;
- implementation head `fe1a981`;
- closure head `a68094e128d2b92f0fd5b350e38512ef6b65cb6b`.

M131 remains residual applicability/primitive authority:

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

- `325 apply`;
- `47 blocked_primitive`;
- `468 not_applicable`.

Full Proposal 170 status remains **partial**.

## Dependency graph

```text
M130 integrated requalification                    [CLOSED — CURRENT RUNTIME AUTHORITY]
  |
  v
M131 residual primitive re-freeze                  [CLOSED AS BLOCKED — 284/88/468]
  |
  +--> M132 combined reduction attempt             [CLOSED AS BLOCKED]
  |      x
  +--> M133 combined close attempt                 [CLOSED AS BLOCKED]
  |
  v
 M135 neutral live quantity + LeaseSet target       [CLOSED AS COMPLETE]
   |
   v
 M136 M132 corrective Reduce*                       [CLOSED AS COMPLETE — 305/67/468]
  |
  v
 M137 M133 corrective Close*                        [CLOSED AS COMPLETE — 319/53/468]
  |
  v
M134 NewDest on proven idle resume                  [CLOSED AS COMPLETE — 325/47/468]
```

## Residual clusters outside the active line

Remain unregistered under M131 authority:

- presentation `UseSSL`;
- `SigType` destination signing;
- `UseOutproxyPlugin`;
- HTTP `SSLProxies` / `JumpList`;
- streaming `Profile`;
- Streamr `ConnectDelay` if still residual;
- `UniqueLocalAddressPerClient`;
- `MultiHoming` / `shouldBundleReplyInfo`;
- encrypted/authenticated LeaseSets.

No successor from these clusters may be smuggled into M134-M137.

## Canonical containment rules

1. Proposal policy stays under `emissary-cli/src/i2pcontrol/**` wherever possible.
2. M135 is an explicit neutral lower-layer exception limited to its named tunnel-pool/destination/LeaseSet owners.
3. M135 authorizes no SAM or I2PControl production change and no matrix promotion.
4. M136 is an explicit neutral SAM idle-policy exception (`sam/session.rs`) plus I2PControl consumer (`backends/runtime/session.rs`, four TCP client allowlists via shared `socks` owner, matrix/ledger/docs); it promotes only the 21 evidence-backed `Reduce*` cells.
5. M137 is an explicit neutral SAM idle-close exception (`sam/session.rs`, `sam/mod.rs`, `lib.rs`) plus I2PControl consumer (`backends/runtime/session.rs`, four TCP client allowlists, `sam_observer.rs`, matrix/ledger/docs); it promotes only the 14 evidence-backed `Close`/`CloseTime` cells.
6. M134 is an I2PControl-only proven-resume exception (`idle_resume.rs`, `client_secret_store.rs`, `backends/runtime/session.rs`, `backends/runtime/client_listener.rs`, `backends/options.rs`, six TCP client allowlists, `production.rs`, `sam_observer.rs`, `server.rs` + `main.rs` composition seams sharing one tracker, `tunnel_manager.rs` guard, matrix/ledger/docs); it promotes only the six evidence-backed `NewDest` cells with no core change.
6. No Proposal-shaped `emissary-core` API is accepted merely to improve matrix counts.
7. Yosemite remains the sole accepted SAM implementation; no parallel raw SAM stack.
8. Exact Y005 remains isolated behind optional `yosemite-i2pcontrol`.
9. No global patch/path/vendor/floating Yosemite dependency.
10. No direct-clearnet fallback, loopback-confinement weakening, TLS verification bypass or LeaseSet security downgrade.
11. No secrets in RPC/log/Debug/planning evidence.
12. No frontend/startup/config rewrite unless separately authorized.
13. External/upstream activity remains read-only.

## Registration rules

1. **No active implementation handoff; the session-lifecycle line is complete
   (M134 closed). Remaining residuals register only under M131 authority with
   independent gates.**
2. M137 registered only because M136 closure explicitly proved its readiness contract (now closed).
3. M134 registered only after successful M137 closure proved an authoritative idle-close/reopen contract (now closed).
4. Material path/architecture deviations require plan amendment before code.
5. Closure evidence, not implementation assertions, determines support/matrix promotion.
6. Active documentation retains partial-support wording until all applicable residuals are resolved and requalified.

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
| M131 | closed as blocked; residual applicability/primitive re-freeze; matrix 284/88/468 |
| M132 | closed as blocked; zero reduction promotions |
| M133 | closed as blocked; zero close promotions |
| M135 | **closed as complete**; neutral live-quantity/LeaseSet primitive, zero promotions, matrix 284/88/468 at closure |
| M136 | **closed as complete**; SAM idle decrease/restore + Proposal Reduce mapping, 21 promotions, matrix 305/67/468 |
| M137 | **closed as complete**; SAM idle close + reasoned termination + Proposal Close mapping, 14 promotions, matrix 319/53/468 at closure (now superseded by M134 325/47/468) |
| M134 | **closed as complete**; NewDest proven-resume + staged secret transaction + one shared successor, six promotions, matrix 325/47/468 |

Historical closure files remain unchanged.