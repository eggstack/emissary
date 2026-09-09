# I2PControl Proposal 170 Post-M146 Corrective Roadmap

Status: **active / partial; M153 is the only registered successor**

This roadmap supersedes the post-M146 execution ordering in:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`;

while preserving those documents as historical planning authority for M140-M146 and the broader full-support objective.

Current repository baseline at roadmap creation:

- `master` after M146 closure: `a0c4a791a6a7a974d34eaf93c45330aafd11116f`;
- last production-bearing commit: `7cbd80a6d72aa07d158ba9dc74f8bbacef767be2`;
- current M095 matrix: `336 apply / 29 blocked_primitive / 475 not_applicable`;
- M146 closed as blocked with zero promotions and zero production delta;
- M139 remains the last whole-surface runtime/security qualification authority but predates M141-M145;
- no residual capability successor is currently registered.

Pinned Proposal/reference authority remains unchanged and read-only:

- Proposal 170 revision `2026-05-20`, status Open;
- Java I2PControl reference head `45bb593000408071dd376b78848fdc246dccd964`;
- Java I2P/I2PTunnel snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`;
- Yosemite optional exact revision `59140a2277bf296928d2e8ce39a148182eeff044`.

## 1. Why this corrective roadmap exists

M140-M145 materially advanced the residual line:

- M140 reclassified seven false blockers to affirmative N/A;
- M141 promoted 2 `UniqueLocalAddressPerClient` cells;
- M142 promoted 2 HTTP `SSLProxies`/`JumpList` cells;
- M143 promoted 1 `Profile:client` cell through a neutral streaming primitive;
- M144 promoted 4 `UseSSL` cells with application-local TLS;
- M145 promoted 2 `MultiHoming` cells through neutral reply LeaseSet bundling.

That moved the matrix from `325/47/468` after M139 to `336/29/475`.

M146 then correctly stopped blocked because no real bounded local outproxy provider exists within the accepted I2P-only security boundary. A dummy provider or alias over existing `ProxyList` would be accept-inert, while direct-clearnet networking is prohibited.

The successful M141-M145 production work now creates a qualification gap before the cryptographic tail:

- M139 is no longer a current-head whole-surface qualification;
- M095 `current_production_head` is stale;
- several historical tests fail on legitimate later matrix deltas;
- M145 required a small no-std/format follow-up commit after its closure;
- M147 itself requires a dedicated signature-domain/security/exact-owner audit before registration.

This roadmap closes those gaps without reopening M146 or weakening containment.

## 2. Current residual inventory

Machine authority at entry:

| Cluster | Cells | Disposition |
|---|---:|---|
| `SigType` | 10 | blocked; destination signature-suite primitive missing |
| `EncryptLeaseSet` | 5 | blocked; encrypted LeaseSet publication/runtime missing |
| `OptionalLookup` | 5 | blocked; blinded/secret lookup runtime missing |
| `LeaseSetClientAuths` | 5 | blocked; authorized-client modes missing |
| `UseOutproxyPlugin` | 4 | blocked by M146; no safe real provider in current architecture |
| **Total** | **29** | |

No planning milestone may reduce this count without real runtime behavior or affirmative N/A evidence.

## 3. Containment policy

The fork remains governed by M061/M062 and M093.

- Proposal/admin/application policy stays under `emissary-cli/src/i2pcontrol/**` wherever possible.
- Neutral core changes require exact-file authorization, independent lower-layer ownership and Proposal-free APIs.
- Broad `crypto/**`, `netdb/**`, `i2np/**`, `destination/**`, `primitives/**` or transport exceptions are prohibited.
- Planning-only M062 entries do not authorize production code.
- No direct-I2P-to-clearnet DNS/TCP path may be introduced to satisfy `UseOutproxyPlugin`.
- No crypto algorithm may silently fall back to Ed25519 or another suite.
- No encrypted/authenticated LeaseSet option may downgrade to ordinary plaintext publication/lookup.
- Secrets/private keys/auth material remain redacted and generation/persistence safe.
- Yosemite remains exact optional Y005; no global/path/floating fork.
- External/upstream interaction remains read-only.

## 4. Corrective dependency graph

```text
M146 UseOutproxyPlugin feasibility              [CLOSED AS BLOCKED — 336/29/475]
  |
  v
M153 post-M146 current-head requalification     [REGISTERED — ZERO PROMOTION]
  |
  v
M154 M147 signature-domain/owner re-freeze      [DEFERRED — ZERO PROMOTION]
  |
  v
M147 neutral destination signature primitive    [DEFERRED — ZERO PROMOTION]
  |
  v
M148 Proposal SigType completion                [DEFERRED — up to 10 promotions]
  |
  v
M149 encrypted LeaseSet runtime                 [DEFERRED — up to 5 promotions]
  |
  v
M150 OptionalLookup runtime                     [DEFERRED — up to 5 promotions]
  |
  v
M151 LeaseSetClientAuths runtime                [DEFERRED — up to 5 promotions]
  |
  v
M152 final residual requalification             [DEFERRED — ZERO PROMOTION]
```

M146's four blocked `UseOutproxyPlugin` cells are a parallel terminal constraint. The crypto/LeaseSet chain may proceed without reopening M146, but M152 cannot claim full Proposal support while those four cells remain blocked.

A future M146 successor requires an explicit architecture/security decision that introduces a genuinely distinct safe provider. This roadmap does not fabricate such a provider or relax the no-direct-clearnet invariant merely to achieve zero blockers.

## 5. M153 — immediate registered corrective

Plan:

- `plans/implementation/i2pcontrol-proposal-170/153-post-m146-current-head-requalification-and-authority-rebase.md`.

Purpose:

- make the current post-M145 production source the new integrated runtime/security baseline;
- repair historical-vs-current test authority so the ordinary suite does not remain red from stale aggregate assertions;
- requalify M140-M146 composition plus M127-M139 security/lifecycle invariants;
- update M095 production-head metadata;
- explicitly absorb the M145 `7cbd80a...` no-std follow-up into accepted production evidence;
- preserve `336/29/475` with zero promotions and zero production changes.

M153 is the only registered plan.

## 6. M154 — signature pre-registration gate

Plan:

- `plans/implementation/i2pcontrol-proposal-170/154-m147-signature-domain-security-and-owner-refreeze.md`.

M154 is required because M147's own plan forbids registration until the exact signature suite/domain, security disposition, persistence implications, dependencies and exact core owners are frozen.

M154 has zero production and zero promotion budget. Its closure chooses:

- amend/register one bounded M147;
- split M147 into smaller infrastructure slices;
- or leave SigType blocked if required algorithms are unsafe/unmaintainable/incompatible.

Only after M153 closes may M154 be registered.

## 7. M147-M151 security-sensitive tail

Existing plans remain deferred:

- M147 neutral destination signature-suite primitive — infrastructure only, zero promotions;
- M148 Proposal `SigType` — up to ten promotions only after real generated identity/signing/persistence support;
- M149 `EncryptLeaseSet` — up to five promotions only after actual encrypted LeaseSet publication/consumption;
- M150 `OptionalLookup` — up to five promotions only after exact blinded/secret lookup with no public downgrade;
- M151 `LeaseSetClientAuths` — up to five promotions only after real PSK/DH/required client-auth semantics and unauthorized-client rejection;
- M152 final whole-surface qualification — zero promotions and no production changes.

The listed maximums are budgets, not promises. Closure evidence determines actual deltas.

## 8. M146 terminal blocker policy

M146 remains immutable blocked evidence.

Current accepted behavior:

- `UseOutproxyPlugin` supplied to any of its four applicable families fails before allocation;
- omitted flag preserves ordinary configured I2P outproxy behavior;
- no dummy registry/provider exists;
- no direct-clearnet fallback exists;
- no support cells promoted.

Do not reopen M146 merely because later crypto work succeeds.

A future provider successor is allowed only if an independently real provider appears or maintainers explicitly change the architecture/security policy. That successor must be separately planned and cannot be hidden inside M147-M152.

## 9. Final-line completion semantics

This corrective line can terminate in two truthful states:

### Full completion

M152 may mark Proposal 170 complete for the pinned revision only when M095 has zero applicable blockers, including resolution of the four M146 cells by a separately accepted provider successor.

### Safe partial completion

If M147-M151 close successfully but M146 remains blocked, M152 must close the residual implementation line as **partial / terminal under current security policy**, with:

- exact remaining four blockers;
- all other applicable cells requalified;
- no full-support claim;
- no fabricated provider;
- explicit statement that completing those four cells requires a future architecture/security decision.

This is a valid completion of the current safe workstream, not full Proposal support.

## 10. Verification policy

Every milestone must preserve the current durable baseline:

- core and CLI checks;
- no-std where touched core participates;
- M061/M062 containment/dependency guards;
- M095/M105 exact matrix/residual guards;
- M127-M129 security regressions;
- lifecycle composition regressions;
- milestone-specific adversarial/runtime tests;
- `git diff --check`;
- clippy/fmt with pre-existing unrelated drift recorded, never normalized opportunistically.

M153 specifically owns repair of stale historical aggregate tests so future broad test failures again carry signal.

## 11. Registration discipline

Per `plans/003-planning-process.md`:

1. M153 is the only registered plan.
2. M154 remains deferred until M153 closes.
3. M147 remains deferred until M154 closes and explicitly amends/registers it or a split successor.
4. M148-M152 remain unregistered behind their hard dependencies.
5. M146 remains closed blocked and is not an active plan.
6. No plan file presence or M062 planning entry authorizes production work.
7. Material architecture/path/dependency changes require plan amendment before coding.

## 12. Exit criteria

This corrective roadmap closes when:

- M153 establishes a current integrated qualification authority;
- M154 resolves M147 readiness truthfully;
- the safe crypto/LeaseSet chain reaches its truthful terminal state;
- M152 performs final whole-surface qualification;
- active planning/docs identify either zero blockers/full support or exact terminal blockers/partial support;
- containment/security invariants remain intact;
- no upstream mutation/contact/submission occurred.
