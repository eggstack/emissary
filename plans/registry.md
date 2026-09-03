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
| Post-M114 corrective line | **active** | `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md` | **M124 blocked on Yosemite Y005** |
| I2PControl containment | accepted authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | M061/M062 regression authority |

## Current production state

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable.
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and cross-book precedence are operational.
- All 12 canonical TunnelManager data planes and seven canonical actions exist.
- All six ClientServicesInfo selectors are operational.
- API 1-only negotiation and M107/M108 managed TLS hardening are operational.
- M109/M115 startup lifecycle and M110/M116 shared-session/destination ownership correctives are closed.
- M119 corrected M118 standby expiry/variance semantics.
- M121 truthfully demoted unsupported `SigType` and `Close`/`CloseTime`/`NewDest` semantics.
- M122 exact-pins Yosemite Y004 `c2db73dba35dd9392947af5c74df29b0b556775f` through the optional I2PControl-only alias; ordinary Yosemite remains registry 0.7.0.
- Current M095 matrix remains `284 apply / 98 blocked_primitive / 458 not_applicable`.
- Full Proposal 170 status remains **partial**.

## Post-M122 corrective findings

### M120 cancellation atomicity

M120 remains historically closed for deterministic preflight and ordinary error rollback, but later review invalidated its stronger cancellation-completeness claim. `commit_server_start()` disarms its drop guard before asynchronous secret/definition durability awaits, and caller cancellation can release the per-name lifecycle lock before terminalization is complete. `discard_sync()` also uses best-effort `try_lock()` cleanup.

**M123** is the corrective owner. Historical M120 closure is not rewritten; M123's closure supersedes only the affected cancellation-atomicity claim.

### Yosemite Y004 auth-mode consistency

Y004's canonical property spelling/value representation remains valid, but later review found that `lease_set_auth_type` and DH/PSK client entries are not cross-field validated. Yosemite can emit security-relevant entries under namespaces the Java reference does not consume for the selected auth branch.

`eggstack/yosemite` registers **Y005** as its sole ready corrective. Current Emissary has no Proposal LeaseSet client-auth mapping, so this is not an active runtime downgrade, but no future M113 successor may build on the known Y004 inconsistency.

## Dependency graph

```text
                           eggstack/yosemite
Y004 canonical LeaseSet transport              [CLOSED / CURRENT EMISSARY PIN]
  |
  v
Y005 auth-mode/type consistency                [READY IN YOSEMITE]
  |
  +-----------------------------------------------------------+
                                                              |
                           eggstack/emissary                  |
M122 Y004 exact-pin adoption                    [CLOSED]       |
  |                                                           |
  v                                                           |
M123 M120 commit-phase cancellation atomicity   [CLOSED]       |
  |                                                           |
  +-------------------------------+---------------------------+
                                  |
                                  v
M124 Y005 exact-pin adoption                    [BLOCKED ON Y005]
  |
  v
focused M113/LeaseSet capability/crypto audit   [FUTURE / BLOCKED ON M124]
  |
  v
remaining residual implementation + new final reclosure       [FUTURE]
```

## Current Emissary handoff — M123

Plan:

- `plans/implementation/i2pcontrol-proposal-170/123-m120-commit-phase-cancellation-atomicity-corrective.md`

Status: **closed**; closure: `plans/closure/i2pcontrol-proposal-170/123-closure.md`.

Baseline:

- `045d1e8b4eba1141d2488882f99c5ce994db91a8`.

Objective:

- guarantee every control-plane server start/restart reaches a terminal committed or exact rolled-back state even if the caller future is cancelled after runtime start;
- retain per-name lifecycle exclusion through terminalization;
- eliminate best-effort staged-secret cleanup as a correctness mechanism;
- preserve M120 validation order, secret confinement and current matrix counts.

M123 is closed. No Emissary implementation handoff is dependency-ready until Yosemite Y005 closes.

## Registered future plan — M124

Plan:

- `plans/implementation/i2pcontrol-proposal-170/124-y005-auth-consistency-pin-adoption.md`

Status: **proposed / blocked** on Yosemite Y005 closure.

Objective:

- advance only the optional `yosemite-i2pcontrol` alias from exact Y004 to the exact reviewed Y005 implementation revision;
- prove Y005's corrected cross-field validation is reachable from the dependency boundary;
- make no Proposal LeaseSet mapping or matrix change.

## Yosemite current handoff

Repository: `eggstack/yosemite`

Plan:

- `plans/implementation/005-y004-leaseset-auth-mode-consistency-corrective.md`

Status: **ready in Yosemite**.

Y005 freezes and enforces the relationship among LeaseSet type, `leaseSetAuthType`, and numbered DH/PSK entries so typed security material cannot be serialized under a branch where the reference silently ignores it. It implements no router cryptography or Emissary policy.

## Residual Proposal state

Current blocked count 98 remains:

- 4 M111 `UseSSL` cells;
- 10 M121-demoted `SigType` cells;
- 63 M112 client proxy/profile/reduction/lifecycle cells, including 18 M121-demoted `Close`/`CloseTime`/`NewDest` cells;
- 21 M113 presentation/routing/LeaseSet cells.

M123, Y005 and M124 are correctness/infrastructure work and do not promote any cell.

The focused M113/LeaseSet capability/crypto-ownership audit remains deferred until M124 closes. Current Emissary `SamSession` still lacks an accepted encrypted/authenticated LeaseSet construction owner; serializer capability alone is not support.

## Recently closed / superseded claims

| Milestone | Disposition |
|---|---|
| M119 | closed |
| M120 | historical closed; cancellation-atomicity claim requires M123 corrective |
| M121 | closed at 284/98/458 |
| M122 | closed at exact Y004 pin; transport only |
| M123 | closed |
| M124 | blocked on Yosemite Y005 |

Historical closure records remain unchanged. Corrective closures supersede only affected claims.

## Registry rules

1. M123 is closed; M124 is the next Emissary handoff once Yosemite Y005 closes.
2. Yosemite Y005 is separately the sole ready handoff in `eggstack/yosemite`.
3. M124 must not execute until Yosemite Y005 closes and this registry explicitly promotes it.
4. Current Emissary pin remains exact Y004 `c2db73d...` until M124; no floating/fork-head dependency is authorized.
5. Matrix remains `284 / 98 / 458`; correctness infrastructure is not capability evidence.
6. No M113/LeaseSet router implementation plan is registered until M124 closes and the focused crypto/ownership audit lands.
7. Keep Proposal policy in `emissary-cli/src/i2pcontrol/**` wherever possible; M123 is I2PControl-only.
8. No global Yosemite patch/replacement/vendor/path dependency is permitted.
9. All external/upstream sources are read-only; no upstream issue/PR/review/submission/merge/adoption/contact/release activity is authorized.
