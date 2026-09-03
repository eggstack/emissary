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
| Post-M114 corrective line | **active** | `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md` | **M126 closed; no dependency-ready successor** |
| I2PControl containment | accepted authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | M061/M062 regression authority |

## Current production state

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable.
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and cross-book precedence are operational according to current closure evidence; M126 independently requalifies that claim at current head.
- All 12 canonical TunnelManager data planes and seven canonical actions exist; M126 independently requalifies production-owner/lifecycle truthfulness for the currently claimed subset.
- All six ClientServicesInfo selectors are operational according to current closure evidence; M126 independently requalifies their live-source/auth boundary.
- API 1-only negotiation and M107/M108 managed TLS hardening are operational according to current closure evidence; M126 re-runs the adversarial boundary.
- M109/M115 startup lifecycle and M110/M116 shared-session/destination ownership correctives are closed.
- M119 corrected M118 standby expiry/variance semantics.
- M121 truthfully demoted unsupported `SigType` and `Close`/`CloseTime`/`NewDest` semantics.
- M122 exact-pinned Yosemite Y004; M124 exact-pins Yosemite Y005 `59140a2277bf296928d2e8ce39a148182eeff044` through the optional I2PControl-only alias; ordinary Yosemite remains registry 0.7.0.
- Current M095 matrix is `284 apply / 96 blocked_primitive / 460 not_applicable` after M125 reclassified two server-role `AllowInternalSSL` cells as not applicable.
- Full Proposal 170 status remains **partial**.

## Post-M122 corrective findings

### M120 cancellation atomicity

M120 remains historically closed for deterministic preflight and ordinary error rollback, but later review invalidated its stronger cancellation-completeness claim. `commit_server_start()` disarms its drop guard before asynchronous secret/definition durability awaits, and caller cancellation can release the per-name lifecycle lock before terminalization is complete. `discard_sync()` also uses best-effort `try_lock()` cleanup.

**M123** is the corrective owner and is closed. Historical M120 closure is not rewritten; M123's closure supersedes only the affected cancellation-atomicity claim.

### Yosemite Y004 auth-mode consistency — resolved by Y005/M124

Y004's canonical property spelling/value representation remains valid, but later review found that `lease_set_auth_type` and DH/PSK client entries are not cross-field validated. Yosemite can emit security-relevant entries under namespaces the Java reference does not consume for the selected auth branch.

`eggstack/yosemite` closed **Y005** at `59140a2277bf296928d2e8ce39a148182eeff044`, and M124 independently reviewed and adopted that exact revision. Current Emissary has no Proposal LeaseSet client-auth mapping, so this was not an active runtime downgrade. No high/medium finding remains open from that chain.

### Post-M125 operational/security qualification gap

M125 closed the focused M113 capability/crypto-ownership audit and corrected the active matrix to `284 / 96 / 460`, but closure history alone is not current-head proof that every existing `apply` cell is operational, authoritative and security-qualified.

**M126** is the evidence-first owner for this requalification. It must independently trace and exercise the currently claimed RouterInfo, AddressBook, TunnelManager and ClientServicesInfo surfaces; re-run authentication/TLS/JSON-RPC/resource adversarial tests; re-check containment; and reconcile active documentation/evidence. Concrete production defects found by M126 require separately registered M127+ correctives rather than opportunistic audit-time fixes.

M126 reconciled the former `AGENTS.md` post-M113 `312/70/458` entry to the current post-M125 `284/96/460` authority.

## Dependency graph

```text
                           eggstack/yosemite
Y004 canonical LeaseSet transport              [CLOSED / HISTORICAL PIN]
  |
  v
Y005 auth-mode/type consistency                [CLOSED]
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
M126 operational/security/spec requalification [CLOSED]
  |
  +--> concrete defect found: register M127+ focused corrective
  |
  +--> no defect/new primitive: retain partial status and 96 blocked residuals
```

The future full-support line remains gated on genuine owners for the residual blocked primitives and a later final full-support reclosure. M126 does not promote blocked cells merely by requalifying the implemented subset.

## Current Emissary handoff — M126

Plan:

- `plans/implementation/i2pcontrol-proposal-170/126-post-m125-operational-security-and-spec-requalification.md`

Status: **ready**.

Planning baseline:

- `685eeeb20f22cdd234e4649c730000d623ad4891`.

Objective:

- independently reconcile the pinned May 20, 2026 Proposal 170 inventory and current `284 / 96 / 460` matrix;
- prove each claimed `apply` family reaches a real authoritative production owner rather than fake/inert/shadow state;
- adversarially requalify authentication, managed TLS, JSON-RPC errors and request/resource bounds;
- requalify AddressBook persistence/confinement, TunnelManager lifecycle/security/cancellation, RouterInfo live-source truthfulness and ClientServicesInfo auth/live-state behavior;
- re-audit the M061/M062 containment boundary;
- reconcile active planning/support documentation, including stale current-count claims;
- register narrowly scoped M127+ corrective plans for any concrete production/security defect instead of hiding fixes inside M126.

M126 is a qualification/corrective-planning milestone. It does not implement the 96 blocked cells, broaden router core, or claim full Proposal 170 support.

## Recently closed plan — M126

Plan:

- `plans/implementation/i2pcontrol-proposal-170/126-post-m125-operational-security-and-spec-requalification.md`

Status: **closed**; closure: `plans/closure/i2pcontrol-proposal-170/126-closure.md`.

M126 qualified the implemented subset at current head with matrix, production-composition,
authenticated live-runtime, persistence/lifecycle, RouterInfo/ClientServicesInfo, and
containment evidence. The partial-support authority remains `284 / 96 / 460`; no M127+ plan
was needed and no future residual implementation became dependency-ready.

## Recently closed plan — M125

Plan:

- `plans/implementation/i2pcontrol-proposal-170/125-m113-capability-crypto-ownership-audit.md`

Status: **closed**; closure: `plans/closure/i2pcontrol-proposal-170/125-closure.md`.

M125 corrected the two server-role `AllowInternalSSL` classifications and confirmed no dependency-ready encrypted LeaseSet, per-client address, or multihoming owner for the remaining M113 cells.

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

Current blocked count 96 remains entering M126:

- 4 M111 `UseSSL` cells;
- 10 M121-demoted `SigType` cells;
- 63 M112 client proxy/profile/reduction/lifecycle cells, including 18 M121-demoted `Close`/`CloseTime`/`NewDest` cells;
- 19 M113 presentation/routing/LeaseSet cells; the two server-role `AllowInternalSSL` cells are not applicable because Proposal 170 places that option under HTTP client filtering.

M123, Y005 and M124 are correctness/infrastructure work and do not promote any cell. M125 reclassified only two misclassified server-role `AllowInternalSSL` cells; it promoted no runtime capability.

M125 found no dependency-ready M113 successor. Current Emissary `SamSession` still lacks an accepted encrypted/authenticated LeaseSet construction owner; serializer capability alone is not support.

M126 may correct these counts only from independent cell-level evidence. It may not manufacture support or applicability to reduce the residual.

## Recently closed / superseded claims

| Milestone | Disposition |
|---|---|
| M119 | closed |
| M120 | historical closed; cancellation-atomicity claim superseded by M123 corrective |
| M121 | historical closed at 284/98/458; current matrix corrected by M125 |
| M122 | closed at exact Y004 pin; transport only |
| M123 | closed |
| M124 | closed at exact Y005 pin; focused M113 capability/crypto audit authorized |
| M125 | closed; corrected two `AllowInternalSSL` classifications and froze remaining M113 blockers |
| M126 | closed; current-head operational/security/spec requalification of the implemented subset |

Historical closure records remain unchanged. Corrective closures supersede only affected claims.

## Registry rules

1. M126 is closed; no dependency-ready Emissary Proposal 170 implementation handoff remains. M123-M125 are also closed.
2. Matrix authority entering M126 is `284 / 96 / 460`; correctness infrastructure, parser acceptance and serializer reachability are not capability evidence.
3. M126 must reproduce or truthfully correct the matrix before relying on it for closure.
4. Concrete implementation/security defects discovered by M126 require separately registered M127+ corrective plans; do not opportunistically hide them inside the qualification milestone.
5. No residual M111/M112/M113 capability implementation is registered until a real canonical owner and exact runtime semantics are dependency-ready.
6. Yosemite Y005 is closed at `59140a2277bf296928d2e8ce39a148182eeff044`; Emissary uses it only through the optional exact I2PControl alias.
7. Keep Proposal policy in `emissary-cli/src/i2pcontrol/**` wherever possible. Any production path outside that boundary requires accepted neutral-owner justification and containment evidence.
8. No global Yosemite patch/replacement/vendor/path dependency is permitted.
9. Active documentation must not claim full Proposal 170 support while applicable cells remain blocked.
10. All external/upstream sources are read-only; no upstream issue/PR/review/submission/merge/adoption/contact/release activity is authorized.
