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
| Post-M114 corrective line | **active** | `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md` | **M125 closed; no successor implementation ready** |
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
- M122 exact-pinned Yosemite Y004; M124 exact-pins Yosemite Y005 `59140a2277bf296928d2e8ce39a148182eeff044` through the optional I2PControl-only alias; ordinary Yosemite remains registry 0.7.0.
- Current M095 matrix is `284 apply / 96 blocked_primitive / 460 not_applicable` after M125 reclassified two server-role `AllowInternalSSL` cells as not applicable.
- Full Proposal 170 status remains **partial**.

## Post-M122 corrective findings

### M120 cancellation atomicity

M120 remains historically closed for deterministic preflight and ordinary error rollback, but later review invalidated its stronger cancellation-completeness claim. `commit_server_start()` disarms its drop guard before asynchronous secret/definition durability awaits, and caller cancellation can release the per-name lifecycle lock before terminalization is complete. `discard_sync()` also uses best-effort `try_lock()` cleanup.

**M123** is the corrective owner. Historical M120 closure is not rewritten; M123's closure supersedes only the affected cancellation-atomicity claim.

### Yosemite Y004 auth-mode consistency — resolved by Y005/M124

Y004's canonical property spelling/value representation remains valid, but later review found that `lease_set_auth_type` and DH/PSK client entries are not cross-field validated. Yosemite can emit security-relevant entries under namespaces the Java reference does not consume for the selected auth branch.

`eggstack/yosemite` closed **Y005** at `59140a2277bf296928d2e8ce39a148182eeff044`, and M124 independently reviewed and adopted that exact revision. Current Emissary has no Proposal LeaseSet client-auth mapping, so this was not an active runtime downgrade. No high/medium finding remains open.

## Dependency graph

```text
                           eggstack/yosemite
Y004 canonical LeaseSet transport              [CLOSED / CURRENT EMISSARY PIN]
  |
  v
Y005 auth-mode/type consistency                [CLOSED IN YOSEMITE]
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
M124 Y005 exact-pin adoption                    [CLOSED]
  |
  v
M125 M113 capability/crypto ownership audit     [CLOSED — 2 CELLS RECLASSIFIED]
  |
  v
remaining residual implementation + new final reclosure       [FUTURE / 96 BLOCKED]
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

M123 is closed. M124 is now closed after Yosemite Y005 closure and independent consumer review.

M125 is closed after the focused M113 capability/crypto-ownership audit. It corrected the
`AllowInternalSSL` server-role classification and confirmed that no safe encrypted LeaseSet,
per-client address, or multihoming owner is available for the remaining M113 cells.

## Recently closed plan — M125

Plan:

- `plans/implementation/i2pcontrol-proposal-170/125-m113-capability-crypto-ownership-audit.md`

Status: **closed**; closure: `plans/closure/i2pcontrol-proposal-170/125-closure.md`.

## Recently closed plan — M124

Plan:

- `plans/implementation/i2pcontrol-proposal-170/124-y005-auth-consistency-pin-adoption.md`

Status: **closed**; closure: `plans/closure/i2pcontrol-proposal-170/124-closure.md`.

Objective:

- advance only the optional `yosemite-i2pcontrol` alias from exact Y004 to the exact reviewed Y005 implementation revision `59140a2277bf296928d2e8ce39a148182eeff044`;
- prove Y005's corrected cross-field validation is reachable from the dependency boundary;
- make no Proposal LeaseSet mapping or matrix change.

## Yosemite current handoff

Repository: `eggstack/yosemite`

Plan:

- `plans/implementation/005-y004-leaseset-auth-mode-consistency-corrective.md`

Status: **closed in Yosemite** at `59140a2277bf296928d2e8ce39a148182eeff044`.

Y005 freezes and enforces the relationship among LeaseSet type, `leaseSetAuthType`, and numbered DH/PSK entries so typed security material cannot be serialized under a branch where the reference silently ignores it. It implements no router cryptography or Emissary policy.

## Residual Proposal state

Current blocked count 96 remains:

- 4 M111 `UseSSL` cells;
- 10 M121-demoted `SigType` cells;
- 63 M112 client proxy/profile/reduction/lifecycle cells, including 18 M121-demoted `Close`/`CloseTime`/`NewDest` cells;
- 19 M113 presentation/routing/LeaseSet cells; the two server-role `AllowInternalSSL` cells
  are not applicable because Proposal 170 places that option under HTTP client filtering.

M123, Y005 and M124 are correctness/infrastructure work and do not promote any cell. M125
reclassifies only the two misclassified server-role `AllowInternalSSL` cells; it promotes no
runtime capability.

M125 found no dependency-ready M113 successor. Current Emissary `SamSession` still lacks an
accepted encrypted/authenticated LeaseSet construction owner; serializer capability alone is
not support.

## Recently closed / superseded claims

| Milestone | Disposition |
|---|---|
| M119 | closed |
| M120 | historical closed; cancellation-atomicity claim requires M123 corrective |
| M121 | historical closed at 284/98/458; current matrix corrected by M125 |
| M122 | closed at exact Y004 pin; transport only |
| M123 | closed |
| M124 | closed at exact Y005 pin; focused M113 capability/crypto audit authorized |
| M125 | closed; corrected two `AllowInternalSSL` classifications and froze remaining M113 blockers |

Historical closure records remain unchanged. Corrective closures supersede only affected claims.

## Registry rules

1. M123, M124 and M125 are closed; no M113 successor implementation is dependency-ready.
2. Yosemite Y005 is closed at `59140a2277bf296928d2e8ce39a148182eeff044`.
3. Current Emissary uses exact Y005 only through the optional I2PControl alias; no floating/fork-head dependency is authorized.
4. Historical closure records remain unchanged; M124 supersedes only the Y004 consumer-pin readiness claim.
5. Matrix is `284 / 96 / 460`; correctness infrastructure is not capability evidence.
6. No M113/LeaseSet router implementation plan is registered because M125 found no safe canonical owner and exact runtime contract.
7. Keep Proposal policy in `emissary-cli/src/i2pcontrol/**` wherever possible; M123 is I2PControl-only.
8. No global Yosemite patch/replacement/vendor/path dependency is permitted.
9. All external/upstream sources are read-only; no upstream issue/PR/review/submission/merge/adoption/contact/release activity is authorized.
