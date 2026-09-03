# I2PControl Proposal 170 — Post-M114 Corrective Roadmap

Status: active corrective workstream (M119/M120/M121/M122 closed; LeaseSet capability audit authorized, not yet registered)

Baseline: `feafc6a1d9650887015a01f87bf21b57a4e92085`

Pinned Proposal 170 revision: `2026-05-20` (Open).

Parent roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Accepted architecture:

- ADR-0001 through ADR-0005;
- M061/M062 containment;
- M093 tunnel security;
- M109-M118 historical closure chain.

External internal-fork dependency:

- `eggstack/yosemite` Y004 corrective roadmap/plan, writes authorized only under that fork's registry.

## 1. Purpose

Resolve post-M114 correctness findings without rewriting historical closures, expanding Proposal scope, or broadly modifying the security-audited Emissary codebase.

M114 remains a valid historical final-reclosure result for its reviewed head: Proposal 170 was partial with 70 applicable blocked cells. This roadmap exists because later review found material defects/ambiguities in one neutral prerequisite (M118), one still-unclosed server start transaction invariant, two support-classification areas (M111/M112), and Yosemite Y003.

The goal is first to restore trustworthy correctness evidence. Full Proposal 170 completion remains governed by the parent roadmap and requires a new final reclosure after all applicable residuals are actually resolved.

## 2. Current findings

### C1 — M118 inbound standby lease expiry

Promoted standby inbound tunnels are republished with a fresh `now + TUNNEL_EXPIRATION` Lease even though their original destruction timer continues from construction. Owner-visible Lease expiration can therefore outlive the actual promoted tunnel.

### C2 — M118 negative variance reference semantics

The current negative-variance sampler is uniformly distributed across offsets while Java reference selection samples magnitude/sign differently. M118 closure claimed exact reference semantics without a deterministic distribution comparison.

### C3 — server start secret mutation before full deterministic validation

`ProductionTunnelManagerControl::start_locked()` prepares/persists server destination identity before backend-specific validation. Unsupported starts can allocate/import/replace private destination state before failing; dynamic failures also lack exact secret/durable-definition rollback.

### C4 — M111 `SigType` truthfulness

I2PControl accepts only canonical string `"7"` while Yosemite can serialize a wider numeric field. Whether ten `SigType` cells should remain `apply` requires explicit Proposal/reference and actual signing-capability evidence.

### C5 — M112 `Close`/`CloseTime`/`NewDest` idle semantics

M112 uses absence of active local TCP handler tasks as its idle signal. Reference `i2cp.closeOnIdle` is an I2P-session idle policy. Equivalence has not been demonstrated, so 18 applied cells require exact proof/correction or demotion.

### C6 — Yosemite Y003 LeaseSet wire semantics

Y003 is not consumed by current Emissary, but its private/signing-key property names, per-client auth namespace/representation, and numeric domains require correction. Yosemite Y004 owns this fix.

## 3. Invariants

All corrective milestones preserve:

- Proposal 170 only; no general I2PControl parity project;
- no protocol aliases/fields/types beyond pinned Proposal/reference needs;
- Proposal-specific business logic stays in `emissary-cli/src/i2pcontrol/**` wherever possible;
- core changes require a neutral canonical owner and separately bounded plan;
- ordinary/non-I2PControl Yosemite dependency remains registry 0.7.0;
- internal fork is consumed only through exact optional `yosemite-i2pcontrol` rev under ADR-0005;
- no accept-inert or fabricated support state;
- unsupported options fail before avoidable allocation;
- no secret leakage or weaker security fallback;
- literal-loopback/proxy/HTTP/IRC/Streamr security boundaries remain intact;
- historical closures are not rewritten to erase defects;
- no upstream review/merge/PR/issue/contact/submission/adoption/release activity.

## 4. Corrective dependency graph

```text
                    eggstack/yosemite
Y003 historical LeaseSet attempt
  |
  v
Y004 canonical LeaseSet wire corrective              [CLOSED IN YOSEMITE at c2db73d]
  |
  +---------------------------------------------------------------+
                                                                   |
                     eggstack/emissary                             |
M114 historical final reclosure                                  |
  |                                                               |
  v                                                               |
M119 M118 standby-expiry/variance corrective          [CLOSED]    |
  |                                                               |
  v                                                               |
M120 server preallocation/secret transaction          [CLOSED]    |
  |                                                               |
  v                                                               |
M121 M111/M112 semantic truthfulness                  [CLOSED 284/98/458]    |
  |                                                               |
  +-------------------------------+-------------------------------+
                                    |
                                    v
M122 corrected Yosemite exact-pin adoption            [CLOSED at Y004 c2db73d]
  |
  v
fresh M113/LeaseSet capability audit + plan           [AUTHORIZED / NOT YET REGISTERED]
  |
  v
remaining residual plans / new final reclosure        [FUTURE]

Only M122's closure is the latest Emissary record. Y004 is closed in the authorized Yosemite fork and adopted by M122. M119, M120, M121, and M122 are closed by `plans/closure/i2pcontrol-proposal-170/119-closure.md`, `plans/closure/i2pcontrol-proposal-170/120-closure.md`, `plans/closure/i2pcontrol-proposal-170/121-closure.md`, and `plans/closure/i2pcontrol-proposal-170/122-closure.md` respectively.

## 5. M119 — neutral tunnel-pool correctness

Plan:

- `plans/implementation/i2pcontrol-proposal-170/119-m118-standby-expiry-and-variance-semantics-corrective.md`.

Status: **closed**.

Owner: `emissary-core/src/tunnel/pool/mod.rs` only, plus tests/containment/planning evidence.

Exit:

- promoted inbound standby expiration never exceeds the tunnel's real original lifetime;
- promotion/failure/shutdown accounting remains bounded and consistent;
- negative variance has an independently frozen reference disposition;
- no Proposal matrix change unless an explicit new semantic failure requires a successor corrective.

## 6. M120 — server start preallocation/secret transactionality

Plan:

- `plans/implementation/i2pcontrol-proposal-170/120-server-start-preallocation-validation-and-secret-transactionality-corrective.md`.

Status: **closed**.

Owner: I2PControl only.

Closure: `plans/closure/i2pcontrol-proposal-170/120-closure.md`.

Exit:

- all deterministic backend/common validation precedes server key generation/import/persistence;
- dynamic failed start restores exact previous secret and durable definition;
- no orphan identity/key or half-committed server state;
- startup-managed ownership remains unchanged.

## 7. M121 — M111/M112 semantic truthfulness

Plan:

- `plans/implementation/i2pcontrol-proposal-170/121-m111-m112-semantic-truthfulness-corrective.md`.

Status: closed (implementation `21f4070`; closure `plans/closure/i2pcontrol-proposal-170/121-closure.md`).

Owner: I2PControl only; no core/Yosemite/crypto change (stop conditions held, demotion path taken).

Exit (achieved):

- `SigType` demoted via Outcome C (10 cells); singleton `{7}` is inert, not configurable support;
- `Close`/`CloseTime`/`NewDest` demoted via §5.2 (18 cells); local TCP-handler-count idle is not reference I2P-session idle and no observation primitive exists;
- M095/M105/ledger/docs mechanically reconciled to `284 / 98 / 458`.

## 8. Yosemite Y004 — canonical LeaseSet transport

Yosemite plan:

- `eggstack/yosemite:plans/implementation/004-y003-leaseset-wire-semantics-corrective.md`.

Status: **closed** (implementation `548c174`; closure `plans/closure/i2pcontrol-proposal-170/122-closure.md`).

Emissary remains on no Y003 surface at any point. Y004 is closed; M122 adopts it.

Y004 corrects generic protocol transport only; it does not establish that Emissary can construct encrypted/authenticated LeaseSets.

## 9. M122 — corrected internal-fork adoption

Plan:

- `plans/implementation/i2pcontrol-proposal-170/122-corrected-yosemite-leaseset-pin-adoption.md`.

Status: **closed** (implementation `548c174`; closure `plans/closure/i2pcontrol-proposal-170/122-closure.md`).

Exit (achieved):

- optional I2PControl alias pins exact reviewed Y004 `c2db73dba35dd9392947af5c74df29b0b556775f`;
- ordinary Yosemite provenance unchanged;
- corrected LeaseSet API/wire reachable via fake-SAM adapter evidence (canonical private/signing keys, one DH + one PSK entry, representative type domains; malformed values reject before wire; material redacted);
- matrix unchanged at `284 / 98 / 458` (infrastructure only, no Proposal promotion).

## 10. Why no M113-successor router implementation plan is registered yet

Y004 resolves only the Yosemite client-to-SAM transport vocabulary, and M122
proves that vocabulary reachable from I2PControl tests without any Proposal
mapping. Current Emissary `SamSession` constructs and signs a normal `LeaseSet2`
locally. The post-M114 audit found no existing `EncryptedLeaseSet2`/client-auth
construction owner that can simply be wired up.

Creating encrypted/blinded/authenticated LeaseSet behavior is security-sensitive router functionality and would be a much larger neutral-core exception than M118. Planning governance requires its exact primitive, cryptographic ownership, secret lifecycle, NetDb publication/query semantics, and containment boundary to be frozen before an implementation plan is dependency-ready.

Therefore this corrective roadmap intentionally does **not** pre-authorize a broad core LeaseSet implementation merely to reduce matrix counts. M122's closure authorizes the focused read-only capability audit described below, but no implementation plan is registered until that audit lands.

After M122 closes, perform a focused read-only capability audit. A new numbered neutral-owner plan is warranted only if it can define:

- exact I2P LeaseSet type(s) required by the remaining M113 cells;
- existing vs missing cryptographic primitives;
- server-only canonical owner in core;
- key/secret source and lifetime without exposing I2PControl types to core;
- exact SAM option mapping;
- no downgrade from requested encrypted/authenticated mode to public LeaseSet;
- bounded client-auth cardinality/material handling;
- NetDb publication/query and interoperability evidence;
- minimal exact production paths.

If those cannot be bounded cleanly, M113 LeaseSet cells remain truthfully blocked.

## 11. Residual Proposal work outside corrective findings

At pre-M121 baseline the matrix was `312 apply / 70 blocked_primitive / 458 not_applicable`:

- 4 M111 `UseSSL` cells;
- 45 M112 proxy/plugin/profile/reduction/Streamr lifecycle cells;
- 21 M113 presentation/routing/LeaseSet cells.

M121 truthfully demotes 28 cells, so the post-M121 baseline is `284 apply / 98 blocked_primitive / 458 not_applicable` (98 = 4 UseSSL + 10 SigType + 63 client + 21 server).

M119, M120, Y004, and M122 are infrastructure/correctness work and do not automatically reduce those counts. M121 may demote already-applied cells if truthfulness requires it.

No plan in this corrective roadmap may mark a residual `apply` solely because a config field or wire serializer exists.

## 12. Verification/closure policy

Each milestone receives its own closure record with requirement-to-evidence mapping, exact commands/outcomes, failure/recovery/contention evidence, security/compatibility review, unresolved severity, changed-path audit, and next-readiness decision.

The final Proposal 170 certification milestone must be a **new numbered final reclosure**, not a rewrite/reopen of M114. It becomes ready only after:

- zero applicable `blocked_primitive`/planned/unknown/unsupported cells;
- no open high/medium Proposal-scoped corrective;
- local runtime evidence;
- reference-router/public-network evidence to the extent required by the parent roadmap.

## 13. External-interaction boundary

Writes are authorized only to `eggstack/emissary` and, under its own registry, `eggstack/yosemite`. All I2P/upstream Emissary/upstream Yosemite repositories and maintainer channels are read-only.

No upstream issue, PR, review, discussion, release, submission, merge/adoption request, contribution package, or maintainer contact is part of this roadmap.