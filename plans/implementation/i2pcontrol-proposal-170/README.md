# Proposal 170 Implementation Handoffs

Status: **partial Proposal 170 support; M153 closed as the current qualification authority; M154 closed (disposition C, M147 path blocked)**.

Pinned Proposal revision: `2026-05-20` (Open).

Current machine state:

- M095: `336 apply / 29 blocked_primitive / 475 not_applicable` across 840 TunnelManager option/family cells;
- remaining blockers: 10 `SigType`, 15 LeaseSet-security (`EncryptLeaseSet`/`OptionalLookup`/`LeaseSetClientAuths`), 4 `UseOutproxyPlugin`;
- M146 is closed as blocked with zero promotions and zero production delta;
- M139 is the historical whole-surface qualification authority, superseded by M153 for current-head purposes because M141-M145 added production behavior afterward;
- M153 is closed as the current runtime/security qualification authority before crypto work;
- M154 is closed (disposition C) and M147 is closed as blocked (path): the ten `SigType` cells are terminal blockers under current security/dependency policy.

Current corrective roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m146-corrective-roadmap.md`.

This roadmap supersedes the post-M146 execution ordering in the earlier residual/full-support roadmaps while preserving them as historical authority for completed work.

## Authority order

1. `plans/000-long-term-specification.md`;
2. `plans/001-terminology-and-domain-model.md`;
3. `plans/002-long-term-roadmap.md`;
4. `plans/003-planning-process.md`;
5. ADR-0001 through ADR-0005;
6. subsystem roadmaps, including the post-M146 corrective roadmap for current execution;
7. `plans/registry.md`;
8. the specific registered implementation plan.

Containment/support evidence remains centered on:

- `061-containment-boundary.toml`;
- `062-dependency-containment.toml`;
- `095-full-support-matrix.toml`;
- `105-residual-option-audit.toml`;
- `110-completion-ledger.toml`.

## Completed residual line through M146

| Milestone | Disposition | Matrix after closure |
|---|---|---:|
| M140 | residual streaming applicability re-freeze; 7 blocked -> N/A, zero promotions | `325/40/475` |
| M141 | `UniqueLocalAddressPerClient` × 2 | `327/38/475` |
| M142 | HTTP `SSLProxies` + `JumpList` × 2 | `329/36/475` |
| M143 | retained `Profile:client` × 1 | `330/35/475` |
| M144 | application/presentation `UseSSL` × 4 | `334/31/475` |
| M145 | `MultiHoming` / `shouldBundleReplyInfo` × 2 | `336/29/475` |
| M146 | `UseOutproxyPlugin` feasibility gate; closed blocked, zero promotions | `336/29/475` |

M145's accepted production state includes follow-up commit `7cbd80a6d72aa07d158ba9dc74f8bbacef767be2`, which corrected a no-std `String` import and formatting after the original M145 closure commit. M153 explicitly requalifies this final production state.

## Registered handoff — M154 (M153 closed)

M153 closure:

- `plans/closure/i2pcontrol-proposal-170/153-closure.md`.

Status: **closed as complete**.

M153 was qualification/test/documentation-only with:

- zero Proposal promotions (matrix unchanged at `336/29/475`);
- zero production Rust/dependency/Yosemite changes;
- corrected M095 `current_production_head` to `7cbd80a6d72aa07d158ba9dc74f8bbacef767be2`;
- historical-vs-current test authority repair with the aggregate owned by the new `m153_post_m146_requalification` guard;
- M127-M145 plus blocked M146 behavior requalified as one current-head surface;
- exact M061/M062 containment and feature-owned dependency isolation re-proved;
- current runtime/security qualification authority rebased from M139 to M153.

## Closed handoff — M154

Plan/closure:

- `154-m147-signature-domain-security-and-owner-refreeze.md`;
- `plans/closure/i2pcontrol-proposal-170/154-closure.md`.

Status: **closed as complete; disposition C — M147 path closed as blocked**.

M154 existed because M147 explicitly cannot be registered until a dedicated audit freezes:

- exact Proposal/reference SigType domain;
- required signature algorithms and security disposition;
- actual Emissary generation/sign/verify/serialization/persistence support per type;
- persistence/migration compatibility;
- exact file owners and dependencies;
- exact-file M061/M062 path budget.

M154 had zero production and zero promotion budget. Its evidence (destination-capable
domain `{0, 1, 2, 3, 7, 11}`; type-0 generation rejected by policy; types 1–3
legacy-only; type 11 missing a maintained primitive; Ed25519-only end-to-end
capability; versioned-storage migration required but not designed) permits neither
a bounded single primitive (A) nor an honest split (B), so M154 registered nothing
and left SigType blocked according to evidence.

## Closed path — M147

Plan/closure:

- `147-neutral-destination-signature-suite-primitive.md` (closed as blocked — path);
- `plans/closure/i2pcontrol-proposal-170/154-closure.md` (disposition C authority).

Status: **closed as blocked (path)**. No production, dependency, or M061/M062 budget
is authorised. Re-opening requires a separate explicit architecture/security decision
and plan. M148 remains deferred behind the unsatisfiable M147 gate (blocked-behind-M147).

## Deferred cryptographic tail

These plans exist but are **not executable** until explicitly registered after their hard dependencies close:

- M147 `147-neutral-destination-signature-suite-primitive.md` — **closed as blocked (path, via M154 disposition C)**; zero promotions, no budget authorised;
- M148 `148-proposal-sigtype-completion.md` — up to 10 real SigType promotions (deferred behind the unsatisfiable M147 gate);
- M149 `149-encrypted-leaseset-runtime-and-encryptleaseset-completion.md` — up to 5 EncryptLeaseSet promotions;
- M150 `150-leaseset-optionallookup-completion.md` — up to 5 OptionalLookup promotions;
- M151 `151-leaseset-client-auths-completion.md` — up to 5 LeaseSetClientAuths promotions;
- M152 `152-final-residual-proposal-170-requalification.md` — final zero-promotion whole-surface qualification.

Maximum promotion budgets are not promises. Runtime evidence decides actual cell disposition.

## M146 blocked policy

M146 is immutable blocked evidence. The four `UseOutproxyPlugin` cells remain unsupported because the current architecture has no real bounded local provider with distinct runtime behavior:

- dummy/registry-only provider: zero support value;
- alias over existing `ProxyList`: accept-inert / semantically collapsed;
- direct OS DNS/TCP provider: prohibited direct-clearnet escape path.

The crypto/LeaseSet tail may proceed without reopening M146, but full Proposal-170 support cannot be claimed while these four applicable blockers remain.

A future provider successor requires a separate explicit architecture/security decision and plan. It may not be hidden inside M147-M152.

## Final-line outcomes

M152 may end the current residual workstream in either truthful state:

- **full support**, only if M095 has zero applicable blockers, including a separately accepted resolution of M146's four cells; or
- **safe partial / terminal under current policy**, if all safe residual crypto/LeaseSet work is complete but `UseOutproxyPlugin` remains blocked, or — as now determined by M154 disposition C — the ten `SigType` cells remain terminal blockers alongside M146's four.

In the latter case active docs must remain explicitly partial and name the exact blockers (currently 10 `SigType` + 4 `UseOutproxyPlugin`, plus any LeaseSet-security cells still blocked at M152).

## Containment rules

- Proposal/admin/application policy stays under `emissary-cli/src/i2pcontrol/**` wherever possible.
- Neutral core seams require exact-file M061/M062 authorization and Proposal-free ownership.
- Planning-only entries in M062 do not authorize production paths.
- No broad `crypto/`, `netdb/`, `i2np/`, `destination/`, `primitives/` or transport waiver.
- No direct-clearnet fallback for proxy/plugin work.
- No crypto/signature/LeaseSet fallback or plaintext downgrade to manufacture support.
- Yosemite remains exact optional Y005 unless a separately accepted Yosemite plan supersedes it.
- External/upstream activity remains read-only; writes stay internal to `eggstack/emissary`.

## Current execution chain

```text
M146 UseOutproxyPlugin feasibility              [CLOSED AS BLOCKED]
  |
  v
M153 current-head integrated requalification    [CLOSED]
  |
  v
M154 signature-domain/security owner re-freeze  [CLOSED — DISPOSITION C]
  |
  v
M147 neutral destination signature primitive    [CLOSED AS BLOCKED (PATH)]
  |
  v
M148 Proposal SigType                            [DEFERRED — BLOCKED BEHIND M147]
  |
  v
M149 -> M150 -> M151 -> M152                     [DEFERRED / UNREGISTERED]
```

Only the next dependency-ready plan is registered at a time. Historical closure files remain unchanged.
