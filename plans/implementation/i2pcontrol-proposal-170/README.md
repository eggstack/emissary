# Proposal 170 Implementation Handoffs

Status: **partial Proposal 170 support; M132/M133 closed as blocked; no active handoff**.

Pinned Proposal revision: `2026-05-20` (Open).

Current authorities:

- runtime/security: M130 closure `plans/closure/i2pcontrol-proposal-170/130-closure.md`;
- residual applicability/primitive map: M131 closure `plans/closure/i2pcontrol-proposal-170/131-closure.md` and `131-residual-primitive-map.toml`;
- current matrix: `284 apply / 88 blocked_primitive / 468 not_applicable`;
- current implementation handoff: none (M132/M133 closed as blocked).

## Authority order

1. `plans/000-long-term-specification.md`;
2. `plans/001-terminology-and-domain-model.md`;
3. `plans/002-long-term-roadmap.md`;
4. `plans/003-planning-process.md`;
5. ADR-0001 through ADR-0005;
6. subsystem roadmaps;
7. `plans/registry.md`;
8. the specific registered plan.

Containment/support evidence remains centered on:

- `061-containment-boundary.toml`;
- `062-dependency-containment.toml`;
- `095-full-support-matrix.toml`;
- `105-residual-option-audit.toml`;
- `110-completion-ledger.toml`.

## M132/M133 closure — no active handoff

Plan:

- `132-neutral-sam-idle-reduction-and-proposal-reduce-completion.md`;
- `133-neutral-sam-idle-close-and-reasoned-termination.md`.

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md`.

Status: **both closed as blocked**.

Closures:

- `plans/closure/i2pcontrol-proposal-170/132-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/133-closure.md`.

M132 targeted one vertical contract slice (SAM/I2CP activity, neutral bounded
live pool target, idle reduction/restore, Proposal `Reduce`/`ReduceCount`/
`ReduceTime` via Yosemite, shared-session semantics, truthful LeaseSet/pool
behavior). It closed as blocked with zero promotions: reference items 9–11
could not be resolved without guessing, the live target plus LeaseSet sync
requires a broad redesign, Yosemite typed reduce fields are dormant, and
Streamr applicability remains ambiguous. The matrix remains
`284 / 88 / 468`. No active handoff exists after M132 closure.

## Closed successor — M133

Plan:

- `133-neutral-sam-idle-close-and-reasoned-termination.md`.

Status: **closed as blocked**; hard dependency on M132 was not satisfied
(M132 provided no stable activity/timer owner).

Closure:

- `plans/closure/i2pcontrol-proposal-170/133-closure.md`.

M133 would have extended the same neutral activity/timer state machine for
`Close`/`CloseTime`, performed real idle session teardown and exposed a
neutral authoritative in-process idle-close reason. It closed as blocked
with zero promotions: without the M132 owner there is no single state
machine to extend, no same-clock close deadline, no authoritative teardown
trigger, and no neutral `IdlePolicy` reason. The 14 `Close`/`CloseTime`
cells remain `blocked_primitive`; `NewDest` remains blocked throughout.
The matrix remains `284 / 88 / 468`. It does not implement `NewDest` and
invents no SAM wire extension.

## Deferred successor — M134

Plan:

- `134-newdest-on-proven-idle-resume.md`.

Status: **deferred / unregistered**; hard-depends on M133 closure (not satisfied — M133 closed as blocked without an authoritative idle-close reason).

M134 keeps destination/key rotation under existing I2PControl secret/session owners. It rotates exactly once on successful resume after a proven idle close, with staged-secret rollback on failure/cancellation. Manual stop/start, restart, process restart and unrelated failure do not rotate. Streamr `NewDest` remains not applicable under M131 authority.

## Current support state

According to current closure authority:

- RouterInfo: 43 additions / 42 available / 1 neutral / 0 unavailable;
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and cross-book precedence operational;
- all 12 canonical TunnelManager data planes and seven actions exist for the claimed subset;
- all six ClientServicesInfo selectors operational;
- M127 finite token lifetime, M128 bounded batch conformance and M129 fail-closed non-loopback TLS are requalified by M130;
- M131 corrected eight false applicability blockers and retained 88 genuine primitive blockers.

Full Proposal 170 support is not claimed.

## Active dependency graph

```text
M130 current runtime/security authority         [CLOSED]
  |
  v
M131 residual primitive re-freeze               [CLOSED AS BLOCKED — 284/88/468]
  |
  v
M132 idle reduction + live pool target          [CLOSED AS BLOCKED — 284/88/468]
  |
  x
M133 idle close + reasoned termination          [CLOSED AS BLOCKED — 284/88/468]
  |
  x
M134 NewDest on proven idle resume              [DEFERRED / UNREGISTERED]
```

No handoff is executable under the current registry after M132/M133 closure.

## Other residual clusters

Remain unregistered under M131 authority:

- presentation `UseSSL`;
- destination `SigType`;
- outproxy provider/plugin integration;
- HTTP `SSLProxies` and `JumpList`;
- streaming `Profile`;
- retained Streamr lifecycle ambiguity such as `ConnectDelay`;
- `UniqueLocalAddressPerClient`;
- `MultiHoming`/`shouldBundleReplyInfo`;
- encrypted/authenticated LeaseSets.

No active lifecycle plan authorizes these areas.

## Containment

Preferred production ownership remains `emissary-cli/src/i2pcontrol/**`.

M132 is the narrowly registered exception for generic lower-layer session/pool behavior and must amend M062 to the exact production diff. M133 closed with no core changes under the same exception rule. Core APIs must remain neutral and contain no Proposal/I2PControl business concepts.

Yosemite remains exact-pinned through the optional `yosemite-i2pcontrol` alias; no global patch, vendoring, path dependency, floating ref or parallel raw SAM stack is permitted.

## Verification baseline

Individual plans refine focused tests. Broad implementation verification includes relevant core/CLI checks and tests, M061/M062 containment, M095/M105 matrix guards, live runtime where applicable, clippy, `cargo fmt --all -- --check`, and `git diff --check`.

Known repository-wide stable/nightly rustfmt drift is recorded rather than normalized through unrelated churn.

## Internal-only rule

All writes remain internal to `eggstack/emissary` unless a separate explicit maintainer directive authorizes another internal target. External I2P/upstream Emissary/upstream Yosemite sources are read-only evidence.

No plan authorizes upstream issue/PR/review/contact/submission/release/merge/adoption activity.