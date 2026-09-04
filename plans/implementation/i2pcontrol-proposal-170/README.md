# Proposal 170 Implementation Handoffs

Status: **partial Proposal 170 support; M132 ready / registered**.

Pinned Proposal revision: `2026-05-20` (Open).

Current authorities:

- runtime/security: M130 closure `plans/closure/i2pcontrol-proposal-170/130-closure.md`;
- residual applicability/primitive map: M131 closure `plans/closure/i2pcontrol-proposal-170/131-closure.md` and `131-residual-primitive-map.toml`;
- current matrix: `284 apply / 88 blocked_primitive / 468 not_applicable`;
- current implementation handoff: M132.

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

## Current registered handoff — M132

Plan:

- `132-neutral-sam-idle-reduction-and-proposal-reduce-completion.md`.

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md`.

Status: **ready / registered**.

M132 implements one vertical contract slice:

- actual SAM/I2CP session activity at the streaming/datagram payload boundary;
- a neutral bounded live active inbound/outbound tunnel-pool target separate from configured/base quantity;
- exact idle reduction and restoration semantics;
- Proposal `Reduce`, `ReduceCount`, `ReduceTime` validation and mapping through the existing Yosemite session-option boundary;
- shared-session compatibility/activity aggregation;
- truthful LeaseSet/pool behavior and generation-local cancellation/restart semantics.

M132 is an explicit neutral lower-layer exception. Its production path budget is limited to existing canonical SAM/destination/tunnel-pool owners plus the I2PControl session adapter. No Cargo/dependency/Yosemite/frontend/startup/NetDb/crypto expansion is authorized.

Starting matrix is `284 / 88 / 468`. M132 may promote up to 21 `Reduce*` cells only where direct reference and end-to-end evidence prove applicability; Streamr cells remain blocked if ambiguity remains.

## Deferred successor — M133

Plan:

- `133-neutral-sam-idle-close-and-reasoned-termination.md`.

Status: **deferred / unregistered**; hard-depends on M132 closure.

M133 extends the same neutral activity/timer state machine for `Close`/`CloseTime`, performs real idle session teardown and exposes a neutral authoritative in-process idle-close reason. It does not implement `NewDest` and may not invent a SAM wire extension.

## Deferred successor — M134

Plan:

- `134-newdest-on-proven-idle-resume.md`.

Status: **deferred / unregistered**; hard-depends on M133 closure.

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
M132 idle reduction + live pool target          [READY / REGISTERED]
  |
  v
M133 idle close + reasoned termination          [DEFERRED / UNREGISTERED]
  |
  v
M134 NewDest on proven idle resume              [DEFERRED / UNREGISTERED]
```

Only M132 is executable under the current registry.

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

M132 is the narrowly registered exception for generic lower-layer session/pool behavior and must amend M062 to the exact production diff. Core APIs must remain neutral and contain no Proposal/I2PControl business concepts.

Yosemite remains exact-pinned through the optional `yosemite-i2pcontrol` alias; no global patch, vendoring, path dependency, floating ref or parallel raw SAM stack is permitted.

## Verification baseline

Individual plans refine focused tests. Broad implementation verification includes relevant core/CLI checks and tests, M061/M062 containment, M095/M105 matrix guards, live runtime where applicable, clippy, `cargo fmt --all -- --check`, and `git diff --check`.

Known repository-wide stable/nightly rustfmt drift is recorded rather than normalized through unrelated churn.

## Internal-only rule

All writes remain internal to `eggstack/emissary` unless a separate explicit maintainer directive authorizes another internal target. External I2P/upstream Emissary/upstream Yosemite sources are read-only evidence.

No plan authorizes upstream issue/PR/review/contact/submission/release/merge/adoption activity.