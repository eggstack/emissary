# I2PControl Proposal 170 — Post-M114 Corrective Roadmap

Status: active corrective workstream; M119-M126 closed; **no dependency-ready successor**

Original corrective baseline: `feafc6a1d9650887015a01f87bf21b57a4e92085`

M123/M124 planning baseline: `045d1e8b4eba1141d2488882f99c5ce994db91a8`

M125 audit baseline: `97083896f6170962a8c9610d056e8fc2dd57646d`

M126 planning baseline: `685eeeb20f22cdd234e4649c730000d623ad4891`

Pinned Proposal 170 revision: `2026-05-20` (Open).

Parent roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Accepted architecture:

- ADR-0001 through ADR-0005;
- M061/M062 containment;
- M093 tunnel security;
- historical M109-M125 closure/corrective chain.

Internal dependency fork:

- `eggstack/yosemite`, governed by its own planning registry and ADR-0005 consumer boundary.

## 1. Purpose

Resolve post-M114 correctness findings without rewriting historical closures, expanding Proposal scope, or broadly modifying the security-audited Emissary codebase.

The workstream prioritizes truthful support, executable current-head evidence and transactional/security invariants over reducing matrix counts. Infrastructure, parser acceptance or serializer capability does not become Proposal support until a real request path has verified runtime effect.

Full Proposal 170 completion still requires genuine owners for all applicable residuals and a later new final full-support reclosure. M126 is instead the current-head requalification gate for the already implemented subset.

## 2. Corrective history through M125

The corrective sequence has closed:

- M119 — M118 standby expiry and negative-variance semantics;
- M120 — deterministic server preflight and ordinary secret/durable rollback, with its cancellation claim later superseded by M123;
- M121 — truthful demotion of unsupported `SigType` and `Close`/`CloseTime`/`NewDest` semantics;
- Yosemite Y004 — canonical LeaseSet wire vocabulary and DH/PSK representation;
- M122 — exact optional I2PControl-only Y004 pin adoption;
- M123 — commit-phase cancellation terminalization/atomicity;
- Yosemite Y005 — LeaseSet auth-mode/type consistency;
- M124 — exact optional I2PControl-only Y005 pin adoption;
- M125 — focused M113 capability/crypto-ownership audit and two-cell `AllowInternalSSL` applicability correction.

M121 established the historical matrix `284 apply / 98 blocked_primitive / 458 not_applicable`.

M125 corrected two server-role `AllowInternalSSL` cells from blocked to not-applicable, producing the current authority:

`284 apply / 96 blocked_primitive / 460 not_applicable`.

M125 found no dependency-ready implementation owner for the remaining M113 presentation/routing/LeaseSet cells.

## 3. Historical corrective findings

### C7 — M120 commit-phase cancellation atomicity — resolved by M123

M120's normal error path was transactional, but its original cancellation-completeness claim did not cover cancellation during asynchronous terminalization after backend runtime start. M123 now owns and closes that invariant without rewriting M120 history.

### C8 — Yosemite Y004 LeaseSet auth-mode/type consistency — resolved by Y005/M124

Y004 fixed canonical transport vocabulary but did not cross-validate the selected auth branch against numbered DH/PSK entries. Yosemite Y005 closed that generic typed-transport defect at `59140a2277bf296928d2e8ce39a148182eeff044`; M124 independently reviewed and adopted that exact revision through the optional I2PControl-only alias.

Current Emissary still has no active Proposal mapping for LeaseSet client-auth options. Y005/M124 therefore improve dependency correctness without promoting Proposal cells.

## 4. Current qualification finding

### C9 — post-M125 current-head operational/security evidence needs independent requalification

M125 was a focused M113 capability/crypto-ownership audit, not a fresh end-to-end requalification of every previously implemented Proposal 170 surface. Historical closures establish strong evidence, but they do not by themselves prove that every current `apply` path at `master` remains:

- wire-correct against the pinned May 20, 2026 proposal;
- protected by the current auth/TLS boundary;
- wired to an authoritative production owner rather than fake/inert/shadow state;
- transactionally truthful for mutations;
- bounded under the accepted TunnelManager/AddressBook security invariants;
- contained under M061/M062 after the full corrective sequence.

M126 owns this evidence gap.

A concrete production/security defect discovered by M126 is not silently patched under the audit. It becomes a separately registered M127+ corrective with explicit owner, paths, invariant, tests and containment budget.

## 5. Invariants

All corrective milestones preserve:

- Proposal 170 only; no general I2PControl parity project;
- Proposal-specific policy stays under `emissary-cli/src/i2pcontrol/**` wherever possible;
- core changes require a neutral canonical owner and a separately reviewed plan;
- ordinary Yosemite dependency remains registry 0.7.0 for non-I2PControl paths;
- internal fork is consumed only through an exact optional `yosemite-i2pcontrol` revision under ADR-0005;
- no `accept_inert`, silent security downgrade, success-before-commit or fabricated support state;
- unsupported options fail before avoidable allocation;
- server secret/key state remains confined and transactionally owned;
- literal-loopback/proxy/HTTP/IRC/Streamr boundaries remain intact;
- historical closures are not rewritten to conceal later defects;
- no upstream issue/PR/review/contact/submission/release/merge/adoption activity.

## 6. Dependency graph

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
M119 standby-expiry/variance corrective         [CLOSED]       |
  |                                                           |
  v                                                           |
M120 server preflight/secret transaction        [HISTORICAL CLOSED; M123 CORRECTIVE]
  |                                                           |
  v                                                           |
M121 semantic truthfulness                      [CLOSED 284/98/458]
  |                                                           |
  v                                                           |
M122 Y004 exact-pin adoption                    [CLOSED]       |
  |                                                           |
  v                                                           |
M123 commit-phase cancellation atomicity        [CLOSED]       |
  |                                                           |
  +-------------------------------+---------------------------+
                                  |
                                  v
M124 Y005 exact-pin adoption                    [CLOSED]
  |
  v
M125 M113 capability/crypto ownership audit     [CLOSED — 284/96/460]
  |
  v
M126 operational/security/spec requalification [CLOSED]
  |
  +--> concrete defect: M127+ focused corrective [REGISTER ONLY WHEN EVIDENCED]
  |
  +--> no defect/new owner: retain partial support / 96 blocked
```

M126 does not make the full-support M114 successor ready. A future final certification remains gated on zero applicable residuals plus fresh interoperability/security evidence.

## 7. M123 — commit-phase cancellation atomicity

Plan:

- `plans/implementation/i2pcontrol-proposal-170/123-m120-commit-phase-cancellation-atomicity-corrective.md`

Status: **closed**; closure: `plans/closure/i2pcontrol-proposal-170/123-closure.md`.

Owner: I2PControl production state only.

M123 guarantees terminal committed-or-rolled-back server lifecycle behavior across caller cancellation and preserves per-name lifecycle exclusion through terminalization. It changed no Proposal support cell.

## 8. Yosemite Y005 and M124 — auth consistency transport adoption

Yosemite plan:

- `eggstack/yosemite:plans/implementation/005-y004-leaseset-auth-mode-consistency-corrective.md`

Status: **closed in Yosemite** at `59140a2277bf296928d2e8ce39a148182eeff044`.

Emissary M124 plan:

- `plans/implementation/i2pcontrol-proposal-170/124-y005-auth-consistency-pin-adoption.md`

Status: **closed**; closure: `plans/closure/i2pcontrol-proposal-170/124-closure.md`.

Y005 freezes the cross-field relationship among LeaseSet type, `leaseSetAuthType`, and numbered DH/PSK entries. M124 independently reviewed and exact-pinned that implementation only through the existing optional I2PControl alias. Neither milestone implements Proposal LeaseSet cryptography or changes M095 counts.

## 9. M125 — M113 capability and crypto-ownership audit

Plan:

- `plans/implementation/i2pcontrol-proposal-170/125-m113-capability-crypto-ownership-audit.md`

Status: **closed**; closure: `plans/closure/i2pcontrol-proposal-170/125-closure.md`.

M125 established that Yosemite Y005 supplies bounded SAM transport serialization and validation for canonical LeaseSet fields, but not Proposal-170 semantics, encrypted-LeaseSet construction, NetDb publication/query ownership, or client-auth key lifecycle. It also confirmed that per-client address allocation and multihoming presentation-routing owners are absent.

The two server-role `AllowInternalSSL` cells were reclassified to `not_applicable` because Proposal 170 places that option under HTTP-client filtering. The authoritative matrix entering M126 is `284 / 96 / 460`.

No M113 successor implementation plan is unblocked.

## 10. M126 — post-M125 operational, security and spec requalification

Plan:

- `plans/implementation/i2pcontrol-proposal-170/126-post-m125-operational-security-and-spec-requalification.md`

Status: **closed**; closure: `plans/closure/i2pcontrol-proposal-170/126-closure.md`.

M126 independently requalifies the current implemented subset rather than assuming historical closure implies current-head correctness.

Primary gates:

1. independently reconstruct and reconcile the pinned Proposal 170 inventory and current matrix;
2. trace every claimed `apply` family through auth/domain/production adapter to an authoritative owner;
3. black-box requalify authentication, managed TLS, JSON-RPC failure semantics and request/resource boundaries;
4. requalify AddressBook persistence/confinement/atomicity and normal-resolver coherence;
5. requalify TunnelManager real runtime lifecycle, rollback, cancellation, admission and unsupported-option fail-before-effect behavior;
6. verify RouterInfo and ClientServicesInfo exact wire semantics plus live-source truthfulness/auth confinement;
7. re-audit all non-I2PControl production paths under M061/M062 containment;
8. reconcile active documentation/evidence, including current matrix counts/status.

M126 independently reproduced the `284 / 96 / 460` matrix, reconciled active `AGENTS.md` guidance,
and corrected the M062 historical allowlist so current M125/M126 planning evidence is classified
as evidence rather than production capability. No production defect or dependency-ready residual
primitive was found.

M126 introduces no residual feature solely to make qualification pass. Concrete production defects require separately registered M127+ plans before implementation.

## 11. LeaseSet capability work remains deferred

The current Yosemite fork can transport canonical LeaseSet settings, but current Emissary `SamSession` still locally constructs a normal LeaseSet2 and has no accepted encrypted/authenticated LeaseSet construction owner.

Do not register an M113-successor implementation merely because Y005/M124 make the client-to-SAM API coherent.

A future neutral-core plan is warranted only if it can freeze:

- exact LeaseSet type(s) required by remaining Proposal cells;
- existing versus missing crypto primitives;
- canonical server-side owner;
- secret/key lifecycle and persistence boundary;
- exact SAM option mapping;
- no downgrade from requested encrypted/authenticated mode;
- bounded client-auth cardinality/material handling;
- NetDb publication/query semantics;
- minimal exact production paths and interoperability evidence.

If those cannot be bounded cleanly, M113 LeaseSet cells remain truthfully blocked.

## 12. Residual Proposal state

Current blocked count 96 entering M126 consists of:

- 4 M111 `UseSSL` cells;
- 10 M121-demoted `SigType` cells;
- 63 M112 client proxy/profile/reduction/lifecycle cells, including 18 M121-demoted `Close`/`CloseTime`/`NewDest` cells;
- 19 M113 presentation/routing/LeaseSet cells; the two server-role `AllowInternalSSL` cells are not applicable under Proposal 170's HTTP-client filtering classification.

M123, Y005 and M124 are correctness/infrastructure milestones and do not reduce these counts. M125 corrected classification only and did not promote runtime capability.

M126 may change this matrix only from independent cell-level evidence. A newly available primitive becomes a separately planned implementation, not an audit-time promotion.

## 13. Verification and closure policy

Every milestone gets a separate closure record containing:

- exact implementation/reviewed commit(s);
- requirement-to-evidence mapping;
- exact verification commands/outcomes;
- failure/cancellation/restart/contention evidence where relevant;
- compatibility/migration/security review;
- changed-path containment audit;
- unresolved findings with severity;
- next-readiness decision;
- internal-only external-interaction attestation.

M126 closes only when its implemented-subset qualification is truthful and every concrete production/security defect has either been ruled out or separately registered for correction. M126 closure does **not** confer full Proposal 170 support while applicable residuals remain blocked.

The final full Proposal 170 certification must be a **new numbered reclosure**, not a rewrite of M114. It becomes ready only after zero applicable residuals, no open high/medium Proposal-scoped corrective, local runtime evidence and required external/reference interoperability evidence.

## 14. Successor-plan policy

Do not pre-register speculative M127+ implementation work.

Register a successor only when M126 produces direct evidence of one of:

- a wire/JSON-RPC conformance defect;
- auth/TLS/resource boundary defect;
- fake/inert/shadow production state;
- success-before-commit mutation semantics;
- AddressBook persistence/confinement/atomicity defect;
- TunnelManager lifecycle/cancellation/resource/security defect;
- RouterInfo/ClientServicesInfo source-truthfulness defect;
- containment regression;
- a previously blocked primitive with a newly available canonical owner and exact runtime semantics.

Each successor must be narrowly scoped and file-specific under `plans/003-planning-process.md`.

## 15. External-interaction boundary

Writes are authorized only to `eggstack/emissary` and, under its own registry, `eggstack/yosemite`. All I2P/upstream Emissary/upstream Yosemite sources and maintainer channels are read-only.

No upstream issue, PR, review, discussion, release, submission, merge/adoption request, contribution package or maintainer contact is part of this roadmap.
