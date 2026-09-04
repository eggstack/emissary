# Emissary Active Planning Registry

This file is the compact control surface for active planning.

Canonical direction:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`

Accepted Proposal 170 architecture decisions:

- ADR-0001 through ADR-0005;
- M061/M062 containment;
- M093 tunnel security.

Pinned Proposal 170 revision: `2026-05-20` (Open).

Authorized internal repositories for this workstream:

- `eggstack/emissary`;
- `eggstack/yosemite` only under ADR-0005 and Yosemite's own registered plans.

All upstream/third-party repositories and maintainer channels remain read-only.

## Active roadmaps

| Subsystem | Status | Roadmap | Current handoff |
|---|---|---|---|
| Proposal 170 full-support completion | active / partial | `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` | historical M114 closed as blocked |
| Post-M114 corrective line | **reopened / active** | `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md` | **M129 ready / registered** |
| I2PControl containment | accepted authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | M061/M062 regression authority |

## Current production/support state

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable according to current closure evidence.
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and cross-book precedence are operational according to current closure evidence.
- All 12 canonical TunnelManager data planes and seven canonical actions exist for the currently claimed subset.
- All six ClientServicesInfo selectors are operational according to current closure evidence.
- Current M095 matrix is `284 apply / 96 blocked_primitive / 460 not_applicable` after M125's two `AllowInternalSSL` applicability corrections; M127 changed no cell.
- Full Proposal 170 status remains **partial**.
- M126 remains historical closure evidence, but its clean shared authentication/TLS/JSON-RPC qualification is superseded for current authority by the concrete C12 finding below until M129-M130 close. C10 is resolved by closed M127 (`plans/closure/i2pcontrol-proposal-170/127-closure.md`) and C11 by closed M128 (`plans/closure/i2pcontrol-proposal-170/128-closure.md`).

## Reopened post-M126 corrective findings

### C10 — authentication token lifetime (resolved)

M127 is closed. Every issued token has finite one-day monotonic validity; expired lookup removes atomically and returns `-32004` on first use after expiry, then `-32003`.

Resolution: **M127 closed** — plan `plans/implementation/i2pcontrol-proposal-170/127-base-auth-token-lifetime-corrective.md`, closure `plans/closure/i2pcontrol-proposal-170/127-closure.md`.

### C11 — JSON-RPC batch conformance (resolved)

M126 proved that top-level arrays do not bypass authentication, but valid JSON-RPC 2.0 batches were blanket-rejected at the time. M128 added bounded batch cardinality (`MAX_BATCH_ELEMENTS = 32`), per-element authentication, notification suppression and no unbounded task fan-out.

Resolution: **M128 closed** — plan `plans/implementation/i2pcontrol-proposal-170/128-json-rpc-batch-conformance-corrective.md`, closure `plans/closure/i2pcontrol-proposal-170/128-closure.md`.

### C12 — non-loopback managed-TLS identity

Managed TLS produces a loopback-only identity (`localhost`, `127.0.0.1`, `::1`) while configuration currently permits non-loopback bind with a warning. The corrective must require complete explicit certificate/key material for every non-loopback bind and reject the invalid configuration before listener/TLS-file side effects.

Owner: **M129** — `plans/implementation/i2pcontrol-proposal-170/129-nonloopback-managed-tls-fail-closed-corrective.md` (ready / registered).

### Integrated current-head requalification

After M127-M129 close, **M130** performs a fresh shared-control-plane plus representative Proposal production requalification and becomes the only milestone in this sequence allowed to restore a clean current-head implemented-subset qualification statement.

M130: `plans/implementation/i2pcontrol-proposal-170/130-post-m127-m129-corrective-requalification.md` — blocked/unregistered on M128-M129 closure (M127-M128 closed).

## Canonical scope correction

The reopened line does **not** implement unrelated base I2PControl methods merely for Proposal 170 completion. `plans/000-long-term-specification.md` explicitly keeps `GetKeys`, `GetRate`, `RouterManager`, `NetworkSetting`, `AdvancedSettings`, and similar unrelated base-method parity outside this workstream.

Shared base behavior is in scope only where required by the implemented extension surface: API-1 authentication/version/token semantics, HTTPS serving, JSON-RPC envelopes/IDs/notifications/batches, and protected dispatch.

## Dependency graph

```text
M126 post-M125 requalification                 [HISTORICAL CLOSED; C10 RESOLVED BY M127, C11 RESOLVED BY M128, C12 OPEN]
M127 token-lifetime corrective                 [CLOSED]
  |
  v
M128 JSON-RPC batch corrective                 [CLOSED]
  |
  v
M129 non-loopback TLS fail-closed corrective   [READY / REGISTERED]
  |
  v
M130 post-corrective requalification           [BLOCKED / UNREGISTERED]
  |
  +--> clean: retain partial 284/96/460 and current-head qualification
  |
  +--> concrete defect: register M131+ focused corrective
```

Only the next dependency-ready plan is registered, consistent with `plans/003-planning-process.md`.

## Current Emissary handoff — M129

Plan:

- `plans/implementation/i2pcontrol-proposal-170/129-nonloopback-managed-tls-fail-closed-corrective.md`

Status: **ready / registered**.

Planning baseline:

- `9948cfd0782a3defbd5f68cf2d4523603bdc7940` for plan creation; M129 implementation/review baseline MUST be reset to the closed-M128 head.

Objective:

- make managed TLS loopback-only and require complete explicit certificate/key configuration for non-loopback binds;
- reject invalid remote/managed configuration before listener/task/managed-file side effects;
- keep every production change under `emissary-cli/src/i2pcontrol/**`;
- change no Proposal 170 matrix cell.

M128 is closed (`plans/closure/i2pcontrol-proposal-170/128-closure.md`, implementation `0ed60eb`) and resolved C11. M129 starts from the closed-M128 head.

## Closed handoff — M128

Plan:

- `plans/implementation/i2pcontrol-proposal-170/128-json-rpc-batch-conformance-corrective.md`

Status: **closed**; closure `plans/closure/i2pcontrol-proposal-170/128-closure.md`, implementation `0ed60eb`.

M128 replaced blanket top-level-array rejection with bounded JSON-RPC 2.0 batch behavior (`MAX_BATCH_ELEMENTS = 32`): per-element authentication with M127 valid/expired/unknown semantics, independent errors/results, exact notification suppression with no-content all-notification batches, zero execution for over-cap batches, no implicit intra-batch token propagation or transaction semantics, and no unbounded task fan-out. Single-request behavior and the Proposal matrix are unchanged. It supersedes only M126's affected batch-conformance claim.

## Closed handoff — M127

Plan:

- `plans/implementation/i2pcontrol-proposal-170/127-base-auth-token-lifetime-corrective.md`

Status: **closed**; closure `plans/closure/i2pcontrol-proposal-170/127-closure.md`, implementation `098c9d1`.

M127 gave every issued token finite one-day monotonic validity, distinguished valid/expired-and-removed/unknown, mapped expiry to `-32004` and later unknown use to `-32003`, and preserved entropy/capacity/conflict/throttle/shutdown/secret bounds with no matrix change. It supersedes only M126's affected authentication-lifetime claim.

## Blocked requalification — M130

Plan:

- `plans/implementation/i2pcontrol-proposal-170/130-post-m127-m129-corrective-requalification.md`

Status: **blocked / unregistered**; hard dependencies M127-M129 closed (M127-M128 closed; M129 open).

M130 freezes the actual post-M129 merged head, recomputes matrix authority, black-box requalifies corrected auth/JSON-RPC/TLS behavior, reruns representative AddressBook/TunnelManager/RouterInfo/ClientServicesInfo evidence, and re-audits containment/dependency isolation.

## Residual Proposal state

Current blocked count remains 96:

- 4 `UseSSL` cells;
- 10 `SigType` cells;
- 63 client proxy/profile/reduction/lifecycle cells, including 18 `Close`/`CloseTime`/`NewDest` cells;
- 19 server presentation/routing/LeaseSet cells.

M127-M130 are correctness/conformance/security work and do not promote these cells. No residual capability implementation is registered until a genuine canonical owner and exact runtime semantics are dependency-ready.

## Yosemite dependency state

Yosemite Y005 remains closed at `59140a2277bf296928d2e8ce39a148182eeff044` and is consumed only through the optional exact `yosemite-i2pcontrol` alias. Ordinary Yosemite remains the registry package for non-I2PControl use.

No reopened plan changes this dependency boundary.

## Recently closed / superseded claims

| Milestone | Disposition |
|---|---|
| M119 | closed |
| M120 | historical closed; cancellation-atomicity claim superseded by M123 |
| M121 | historical closed at 284/98/458; current matrix corrected by M125 |
| M122 | closed at exact Y004 pin; transport only |
| M123 | closed |
| M124 | closed at exact Y005 pin |
| M125 | closed; corrected two `AllowInternalSSL` classifications |
| M126 | historical closed; current shared auth/TLS/JSON-RPC clean-qualification claim superseded pending M129-M130 (C10 resolved by M127, C11 resolved by M128) |
| M127 | closed; finite one-day token lifetime, expired/unknown distinction, `-32004`/`-32003` mapping; matrix unchanged |
| M128 | closed; bounded batch conformance (`MAX_BATCH_ELEMENTS = 32`), per-element auth, notification/no-content rules; matrix unchanged |

Historical closure records remain unchanged. Corrective closures supersede only affected claims.

## Registry rules

1. M129 is the only currently registered Proposal 170 implementation handoff.
2. M127-M128 are closed; close M129 before promoting M130.
3. Matrix authority remains `284 / 96 / 460`; shared-control-plane corrective work is not capability evidence.
4. Keep Proposal policy in `emissary-cli/src/i2pcontrol/**` wherever possible. Any production path outside that boundary requires accepted neutral-owner justification and containment evidence.
5. No unrelated base-I2PControl method parity is authorized by this corrective line.
6. No residual M111/M112/M113 capability implementation is registered until a real canonical owner and exact runtime semantics are dependency-ready.
7. No global Yosemite patch/replacement/vendor/path dependency is permitted.
8. Active documentation must not claim full Proposal 170 support while applicable cells remain blocked.
9. Concrete independent defects discovered by M127-M130 become separately numbered M131+ correctives; do not broaden an active milestone.
10. All external/upstream sources are read-only; no upstream issue/PR/review/submission/merge/adoption/contact/release activity is authorized.
