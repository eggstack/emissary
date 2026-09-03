# I2PControl Proposal 170 — Post-M114 Corrective Roadmap

Status: active corrective workstream; M119-M124 closed; focused M113 capability/crypto-ownership audit authorized but not yet registered

Original corrective baseline: `feafc6a1d9650887015a01f87bf21b57a4e92085`

Current planning baseline for M123/M124: `045d1e8b4eba1141d2488882f99c5ce994db91a8`

Pinned Proposal 170 revision: `2026-05-20` (Open).

Parent roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Accepted architecture:

- ADR-0001 through ADR-0005;
- M061/M062 containment;
- M093 tunnel security;
- historical M109-M122 closure/corrective chain.

Internal dependency fork:

- `eggstack/yosemite`, governed by its own planning registry and ADR-0005 consumer boundary.

## 1. Purpose

Resolve post-M114 correctness findings without rewriting historical closures, expanding Proposal scope, or broadly modifying the security-audited Emissary codebase.

The workstream prioritizes truthful support and transactional/security invariants over reducing matrix counts. Infrastructure or serializer capability does not become Proposal support until a real request path has verified runtime effect.

Full Proposal 170 completion still requires a new final reclosure after all applicable residuals are actually resolved.

## 2. Corrective history through M122

The first corrective sequence closed:

- M119 — M118 standby expiry and negative-variance semantics;
- M120 — deterministic server preflight and ordinary secret/durable rollback;
- M121 — truthful demotion of unsupported `SigType` and `Close`/`CloseTime`/`NewDest` semantics;
- Yosemite Y004 — canonical LeaseSet wire vocabulary and DH/PSK representation;
- M122 — exact optional I2PControl-only Y004 pin adoption.

M121 established the current authoritative matrix:

`284 apply / 98 blocked_primitive / 458 not_applicable`.

Later review found two additional correctness gaps that this roadmap now owns.

## 3. New findings

### C7 — M120 commit-phase cancellation atomicity

M120's normal error path is transactional, but `commit_server_start()` disarms its cancellation guard before asynchronous secret durability and definition/public-destination persistence.

If the caller future is dropped during that terminalization window:

- rollback code is not guaranteed to run;
- the caller-owned per-name lifecycle lock can be released before terminal state;
- fresh/replacement secret durability can diverge from the durable definition/runtime state.

Additionally, `ServerDestinationStore::discard_sync()` uses `try_lock()`, making cancellation cleanup best-effort under contention.

M123 owns this corrective.

### C8 — Yosemite Y004 LeaseSet auth-mode/type consistency

Y004 correctly fixed property spelling, client-auth representation and numeric domains, but it validates auth fields independently.

The typed API can currently serialize DH and/or PSK numbered entries without enforcing the branch selected by `i2cp.leaseSetAuthType`. The Java reference consumes:

- DH namespace for auth type 1;
- PSK namespace for auth type 2;
- neither namespace in the no-auth branch.

A typed library must not serialize security material that the effective reference behavior silently ignores.

Yosemite Y005 owns the generic API-to-wire corrective. Current Emissary has no active Proposal mapping for LeaseSet client-auth options, so this is a dependency correctness prerequisite rather than a current runtime downgrade.

## 4. Invariants

All corrective milestones preserve:

- Proposal 170 only; no general I2PControl parity project;
- Proposal-specific policy stays under `emissary-cli/src/i2pcontrol/**` wherever possible;
- core changes require a neutral canonical owner and a separately reviewed plan;
- ordinary Yosemite dependency remains registry 0.7.0 for non-I2PControl paths;
- internal fork is consumed only through an exact optional `yosemite-i2pcontrol` revision under ADR-0005;
- no `accept_inert`, silent security downgrade or fabricated support state;
- unsupported options fail before avoidable allocation;
- server secret/key state remains confined and transactionally owned;
- literal-loopback/proxy/HTTP/IRC/Streamr boundaries remain intact;
- historical closures are not rewritten to conceal later defects;
- no upstream issue/PR/review/contact/submission/release/merge/adoption activity.

## 5. Dependency graph

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
focused M113/LeaseSet capability/crypto audit   [AUTHORIZED / NOT YET REGISTERED]
  |
  v
residual implementations + new final reclosure [FUTURE]
```

M123 is closed. Yosemite Y005 is closed, and M124 has independently adopted its exact reviewed
revision. The focused M113 capability/crypto-ownership audit is now authorized to proceed as
read-only planning work, but no implementation plan is dependency-ready until that audit lands.

## 6. M123 — commit-phase cancellation atomicity

Plan:

- `plans/implementation/i2pcontrol-proposal-170/123-m120-commit-phase-cancellation-atomicity-corrective.md`

Status: **closed**; closure: `plans/closure/i2pcontrol-proposal-170/123-closure.md`.

Owner: I2PControl production state only.

Primary paths:

- `emissary-cli/src/i2pcontrol/production.rs`;
- `emissary-cli/src/i2pcontrol/server_secret_store.rs` where deterministic staged-state cleanup requires it.

Exit conditions:

- every server start/restart reaches exactly committed or rolled-back state even if caller cancellation occurs after backend runtime start;
- per-name lifecycle exclusion remains owned through terminalization;
- staged-secret cleanup is deterministic rather than `try_lock()` best effort;
- fresh/replacement/existing server cases have deterministic cancellation-boundary tests plus reload evidence;
- matrix remains `284 / 98 / 458`.

M123 does not change Proposal option support, Yosemite, Cargo dependencies, router core or startup-managed tunnels.

## 7. Yosemite Y005 — LeaseSet auth consistency

Yosemite plan:

- `eggstack/yosemite:plans/implementation/005-y004-leaseset-auth-mode-consistency-corrective.md`

Status: **ready in Yosemite**.

Y005 independently freezes the cross-field relationship among:

- LeaseSet type/applicability;
- `leaseSetAuthType`;
- numbered DH entries;
- numbered PSK entries.

It rejects typed combinations whose security-relevant entries would be ignored by the reference path, while preserving canonical Y004 names, validation bounds, redaction and default wire.

Y005 implements no LeaseSet cryptography or Emissary policy.

## 8. M124 — exact Y005 adoption

Plan:

- `plans/implementation/i2pcontrol-proposal-170/124-y005-auth-consistency-pin-adoption.md`

Status: **closed**; closure: `plans/closure/i2pcontrol-proposal-170/124-closure.md`.

M124 completed the following:

- reviewed the exact Y004→Y005 Yosemite diff;
- advanced the optional `yosemite-i2pcontrol` `rev` to the exact reviewed Y005 implementation SHA;
- updated the corresponding lock/containment evidence;
- proved representative Y005 acceptance/rejection at the dependency boundary.

M124 did not map Proposal LeaseSet fields and did not change M095 counts.

## 9. LeaseSet capability work remains deferred

The current Yosemite fork can transport canonical LeaseSet settings, but current Emissary `SamSession` still locally constructs a normal LeaseSet2 and has no accepted encrypted/authenticated LeaseSet construction owner.

Do not register an M113-successor implementation merely because Y005/M124 make the client-to-SAM API coherent.

With M124 closed, perform the already-authorized focused read-only capability/crypto-ownership audit. A new neutral-core plan is warranted only if it can freeze:

- exact LeaseSet type(s) required by remaining Proposal cells;
- existing vs missing crypto primitives;
- canonical server-side owner;
- secret/key lifecycle and persistence boundary;
- exact SAM option mapping;
- no downgrade from requested encrypted/authenticated mode;
- bounded client-auth cardinality/material handling;
- NetDb publication/query semantics;
- minimal exact production paths and interoperability evidence.

If those cannot be bounded cleanly, M113 LeaseSet cells remain truthfully blocked.

## 10. Residual Proposal state

Current blocked count 98 consists of:

- 4 M111 `UseSSL` cells;
- 10 M121-demoted `SigType` cells;
- 63 M112 client proxy/profile/reduction/lifecycle cells, including 18 M121-demoted `Close`/`CloseTime`/`NewDest` cells;
- 21 M113 presentation/routing/LeaseSet cells.

M123, Y005 and M124 are correctness/infrastructure milestones and do not reduce these counts.

## 11. Verification and closure policy

Every milestone gets a separate closure record containing:

- implementation commit(s);
- requirement-to-evidence mapping;
- exact verification commands/outcomes;
- failure/cancellation/restart/contention evidence where relevant;
- compatibility/migration/security review;
- changed-path containment audit;
- unresolved findings with severity;
- next-readiness decision;
- internal-only external-interaction attestation.

The final Proposal 170 certification must be a **new numbered reclosure**, not a rewrite of M114. It becomes ready only after zero applicable residuals, no open high/medium Proposal-scoped corrective, local runtime evidence and required external/reference interoperability evidence.

## 12. External-interaction boundary

Writes are authorized only to `eggstack/emissary` and, under its own registry, `eggstack/yosemite`. All I2P/upstream Emissary/upstream Yosemite sources and maintainer channels are read-only.

No upstream issue, PR, review, discussion, release, submission, merge/adoption request, contribution package or maintainer contact is part of this roadmap.
