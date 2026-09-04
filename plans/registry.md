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
| Post-M114 corrective line | **reopened / active** | `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md` | **M127 ready / registered** |
| I2PControl containment | accepted authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | M061/M062 regression authority |

## Current production/support state

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable according to current closure evidence.
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and cross-book precedence are operational according to current closure evidence.
- All 12 canonical TunnelManager data planes and seven canonical actions exist for the currently claimed subset.
- All six ClientServicesInfo selectors are operational according to current closure evidence.
- Current M095 matrix is `284 apply / 96 blocked_primitive / 460 not_applicable` after M125's two `AllowInternalSSL` applicability corrections.
- Full Proposal 170 status remains **partial**.
- M126 remains historical closure evidence, but its clean shared authentication/TLS/JSON-RPC qualification is superseded for current authority by the concrete C10-C12 findings below until M127-M130 close.

## Reopened post-M126 corrective findings

### C10 — authentication token lifetime

Current `TokenService` stores membership only and has no finite expiry state. The RPC layer defines standard `TOKEN_EXPIRED` (`-32004`) behavior, but production validation cannot reach that path. This is an authentication conformance/security-lifetime defect missed by M126.

Owner: **M127** — `plans/implementation/i2pcontrol-proposal-170/127-base-auth-token-lifetime-corrective.md`.

### C11 — JSON-RPC batch conformance

M126 proved that top-level arrays do not bypass authentication, but valid JSON-RPC 2.0 batches are still blanket-rejected. The corrective must add bounded batch cardinality, per-element authentication, notification suppression and no unbounded task fan-out.

Owner: **M128** — written but queued/unregistered until M127 closes.

### C12 — non-loopback managed-TLS identity

Managed TLS produces a loopback-only identity (`localhost`, `127.0.0.1`, `::1`) while configuration currently permits non-loopback bind with a warning. The corrective must require complete explicit certificate/key material for every non-loopback bind and reject the invalid configuration before listener/TLS-file side effects.

Owner: **M129** — written but queued/unregistered behind M128.

### Integrated current-head requalification

After M127-M129 close, **M130** performs a fresh shared-control-plane plus representative Proposal production requalification and becomes the only milestone in this sequence allowed to restore a clean current-head implemented-subset qualification statement.

M130: `plans/implementation/i2pcontrol-proposal-170/130-post-m127-m129-corrective-requalification.md` — blocked/unregistered on M127-M129 closure.

## Canonical scope correction

The reopened line does **not** implement unrelated base I2PControl methods merely for Proposal 170 completion. `plans/000-long-term-specification.md` explicitly keeps `GetKeys`, `GetRate`, `RouterManager`, `NetworkSetting`, `AdvancedSettings`, and similar unrelated base-method parity outside this workstream.

Shared base behavior is in scope only where required by the implemented extension surface: API-1 authentication/version/token semantics, HTTPS serving, JSON-RPC envelopes/IDs/notifications/batches, and protected dispatch.

## Dependency graph

```text
M126 post-M125 requalification                 [HISTORICAL CLOSED; C10-C12 FOUND LATER]
  |
  v
M127 token-lifetime corrective                 [READY / REGISTERED]
  |
  v
M128 JSON-RPC batch corrective                 [QUEUED / UNREGISTERED]
  |
  v
M129 non-loopback TLS fail-closed corrective   [QUEUED / UNREGISTERED]
  |
  v
M130 post-corrective requalification           [BLOCKED / UNREGISTERED]
  |
  +--> clean: retain partial 284/96/460 and current-head qualification
  |
  +--> concrete defect: register M131+ focused corrective
```

Only the next dependency-ready plan is registered, consistent with `plans/003-planning-process.md`.

## Current Emissary handoff — M127

Plan:

- `plans/implementation/i2pcontrol-proposal-170/127-base-auth-token-lifetime-corrective.md`

Status: **ready**.

Planning baseline:

- `9948cfd0782a3defbd5f68cf2d4523603bdc7940`.

Objective:

- give every issued authentication token finite in-process validity;
- use a deterministic/testable monotonic lifetime model;
- distinguish valid, expired-and-removed and unknown credentials;
- map expiry to existing `-32004 TOKEN_EXPIRED` and later unknown use to `-32003`;
- preserve token entropy, capacity, conflict rejection, throttle, shutdown clearing and secret safety;
- keep every production change under `emissary-cli/src/i2pcontrol/**`;
- change no Proposal 170 matrix cell.

M127 supersedes only M126's affected authentication-lifetime qualification claim. Historical M126 closure is not rewritten.

## Queued handoff — M128

Plan:

- `plans/implementation/i2pcontrol-proposal-170/128-json-rpc-batch-conformance-corrective.md`

Status: **queued / unregistered** until M127 closes.

M128 adds bounded valid JSON-RPC batch handling with independent auth/errors/results, exact notification suppression, zero execution for over-cap batches, and no per-element unbounded task fan-out. Single-request behavior remains compatible.

## Queued handoff — M129

Plan:

- `plans/implementation/i2pcontrol-proposal-170/129-nonloopback-managed-tls-fail-closed-corrective.md`

Status: **queued / unregistered** behind M128.

M129 makes managed TLS loopback-only and requires complete explicit certificate/key configuration for non-loopback binds. It adds no automatic remote SAN, trust, mTLS or core/router machinery.

## Blocked requalification — M130

Plan:

- `plans/implementation/i2pcontrol-proposal-170/130-post-m127-m129-corrective-requalification.md`

Status: **blocked / unregistered**; hard dependencies M127-M129 closed.

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
| M126 | historical closed; current shared auth/TLS/JSON-RPC clean-qualification claim superseded pending M127-M130 |

Historical closure records remain unchanged. Corrective closures supersede only affected claims.

## Registry rules

1. M127 is the only currently registered Proposal 170 implementation handoff.
2. Close M127 before promoting M128; close M128 before promoting M129; close M129 before promoting M130.
3. Matrix authority remains `284 / 96 / 460`; shared-control-plane corrective work is not capability evidence.
4. Keep Proposal policy in `emissary-cli/src/i2pcontrol/**` wherever possible. Any production path outside that boundary requires accepted neutral-owner justification and containment evidence.
5. No unrelated base-I2PControl method parity is authorized by this corrective line.
6. No residual M111/M112/M113 capability implementation is registered until a real canonical owner and exact runtime semantics are dependency-ready.
7. No global Yosemite patch/replacement/vendor/path dependency is permitted.
8. Active documentation must not claim full Proposal 170 support while applicable cells remain blocked.
9. Concrete independent defects discovered by M127-M130 become separately numbered M131+ correctives; do not broaden an active milestone.
10. All external/upstream sources are read-only; no upstream issue/PR/review/submission/merge/adoption/contact/release activity is authorized.
