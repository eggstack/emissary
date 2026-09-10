# I2PControl Proposal 170 Full-Support Completion Roadmap

Status: **active / partial; M152 historical safe-partial qualification, post-M152 corrective line active**

Pinned Proposal authority: revision `2026-05-20`, status Open.

Current machine authority: M095 `336 apply / 29 blocked_primitive / 475 not_applicable` across 840 TunnelManager option/family cells. Its cell dispositions remain authoritative; its `current_production_head` metadata is stale and is scheduled for refresh by M165 after the production-bearing M163/M164 correctives.

Current whole-surface qualification ancestry: M152 with M153 ancestry; M152 is immutable historical evidence for the M162 head, not the final post-corrective authority. M139 remains the historical post-lifecycle qualification predecessor to M153.

Historical promotion lineage: M141 `UniqueLocalAddressPerClient` ×2; M142 `SSLProxies`/`JumpList` ×2; M143 retained `Profile:client`; M144 `UseSSL` ×4; M145 `MultiHoming` ×2.

Current execution authority:

- `plans/subsystems/i2pcontrol-proposal-170-post-m152-blocked-state-corrective-roadmap.md`;
- M163 is closed and M164 is the sole registered handoff;
- M165 remains deferred until M164 closes.

## 1. Current residual inventory

| Cluster | Cells | Current disposition |
|---|---:|---|
| `SigType` | 10 | blocked by M154/M147 configurable-destination path |
| `EncryptLeaseSet` | 5 | blocked by M161-B + M162 integration gaps |
| `OptionalLookup` | 5 | blocked; neutral M158 primitive exists but Proposal integration remains incomplete |
| `LeaseSetClientAuths` | 5 | blocked; neutral M159/M160 primitives exist but Proposal integration remains incomplete |
| `UseOutproxyPlugin` | 4 | blocked by M146 |
| **Total** | **29** | |

## 2. Accepted lineage

- M140-M145 completed the safe residual streaming/proxy/presentation/reply-bundling work, reaching `336/29/475`.
- M146 closed UseOutproxyPlugin blocked.
- M153 requalified the then-current production head.
- M154 closed configurable Destination SigType blocked.
- M155 re-froze LeaseSet-security semantics/owners.
- M156 implemented narrow Red25519/blinding.
- M157 implemented modern type-5/no-auth encrypted-LS2 publication/storage verification/UTC rollover.
- M158 implemented standard lookup-secret contribution and encrypted-service extended B32.
- M159 implemented neutral standard PSK client authorization.
- M160 implemented neutral standard DH (X25519) client authorization.
- M156-M160 all closed with zero Proposal promotions.
- M161 closed legacy AES/LS1 outcome B: valid but blocked, no LS1 resurrection.
- M162 closed as blocked Proposal integration with zero promotions.
- M152 closed safe-partial on the M162 head; later review found remaining correctness/security defects now assigned to M163/M164.

Historical closures remain immutable.

## 3. Current corrective chain

```text
M155 semantic/owner refreeze                   [CLOSED]
  -> M156 Red25519/blinding                    [CLOSED]
  -> M157 modern Encrypted LS2                 [CLOSED]
  -> M158 lookup-secret/blinded address        [CLOSED]
  -> M159 PSK client authorization             [CLOSED]
  -> M160 DH/X25519 authorization              [CLOSED]
  -> M161 legacy AES/LS1 feasibility           [CLOSED; OUTCOME B]
  -> M162 Proposal field integration           [CLOSED; BLOCKED INTEGRATION]
  -> M152 historical requalification           [CLOSED; SAFE PARTIAL]
  -> M163 blocked LeaseSet durable/history fix [CLOSED]
  -> M164 SAM invalid-command secret redaction [REGISTERED / DEPENDENCY-READY]
  -> M165 post-corrective requalification      [DEFERRED]
```

M149-M151 are superseded historical drafts and must not be executed. M147/M148 remain blocked under M154-C. M146 remains blocked.

## 4. M163 corrective boundary

M163 fixes a control-plane truthfulness/security defect, not a Proposal capability gap.

Exact production paths:

1. `emissary-cli/src/i2pcontrol/tunnel_manager.rs`;
2. `emissary-cli/src/i2pcontrol/stores/tunnel_store.rs`;
3. `emissary-cli/src/i2pcontrol/stores/generation_store.rs`.

Required behavior:

- blocked `EncryptLeaseSet`, `OptionalLookup`, and `LeaseSetClientAuths` fail Create/Edit before durable mutation;
- M162 backend/session fail-before-allocation gates remain defense in depth;
- historical M162-era typed state is sanitized through a crash-resumable migration;
- at least two clean fallback generations are created before contaminated history is removed;
- contaminated-history purge is explicit/fail-closed and directory-synced rather than ordinary best-effort cleanup;
- interrupted pending scrubs fail load before StartOnLoad and resume on next load;
- no core/Yosemite/dependency/server-secret-store change;
- zero Proposal promotion/demotion.

## 5. M164 corrective boundary

M164 addresses the independent neutral SAM log exposure discovered during post-M152 review.

Exact production path when registered:

- `emissary-core/src/sam/socket.rs`.

Current parser-rejection tracing logs the complete rejected command. Because standard SAM/I2CP LeaseSet options may carry lookup/PSK/DH secret material, M164 must omit rejected payload content wholesale and log only safe structural metadata. No parser/session semantic change, dependency/Yosemite/I2PControl production change, or Proposal promotion is allowed.

The path is already an exact M061 neutral owner, so no broad source-boundary waiver should be required.

## 6. M165 final requalification

M165 is zero-production/zero-promotion and hard-depends on clean M163+M164 closures.

It must:

- set M095 `current_production_head` to the actual last production-bearing M164 closure commit;
- mechanically recompute `336/29/475` and exact blocked identities;
- rerun whole-surface behavioral/security/containment qualification through M164;
- prove no blocked LeaseSet durable effect/history residue and no malformed-SAM secret log echo;
- declare safe-partial current-head authority only if no high/medium defect remains.

## 7. Closed M160 boundary

M160 realized neutral DH infrastructure over exactly:

1. `emissary-core/src/crypto/els2.rs`;
2. `emissary-core/src/destination/lease_set.rs`;
3. `emissary-core/src/sam/parser.rs`;
4. `emissary-core/src/sam/session.rs`.

All are exact realized M061 owners. M160 made no new source-boundary waiver and no dependency/Cargo/lockfile/Yosemite/I2PControl production-source change.

M160 implemented standard DH type-5 authorization, adopted the pinned 4096-byte encrypted-data ceiling as the allocation/O(N) bound, used `i2cp.leaseSetAuthType=1`, required the base X25519 private key, preserved bounded contiguous indexed client keys including duplicates, rejected all-zero shared secrets, and kept key material out of generic debug-capable state.

M159 provides the analogous PSK infrastructure with `authType=2` and the same 4096-byte bound. Both remain neutral infrastructure with zero Proposal promotions.

## 8. Field-completeness rules

- Neutral infrastructure has zero Proposal support value by itself.
- `OptionalLookup` promotes only when its complete Proposal administrative/runtime/persistence contract is operational and interoperable.
- `LeaseSetClientAuths` promotes only when the complete PSK/DH list contract is bounded, persistent, restart-safe and interoperable across all five server families.
- `EncryptLeaseSet` promotes only when every valid enum value is operational for the family. If legacy AES remains valid and unsupported, all five cells remain blocked.
- Parser/serializer/persistence reachability alone is not support.
- Blocked values must not be accepted as durable inert configuration.

## 9. Planning ceilings

From `336/29/475`:

- complete OptionalLookup + LeaseSetClientAuths with EncryptLeaseSet still blocked: at most `346/19/475`;
- complete EncryptLeaseSet enum too: at most `351/14/475`.

SigType ×10 and UseOutproxyPlugin ×4 remain independent blockers. These are ceilings, not claims.

## 10. Containment/security rules

Accepted authority remains ADR-0001 through ADR-0005, M061/M062 and M093.

- Proposal/admin policy stays in I2PControl wherever possible.
- Neutral core seams require exact owners and Proposal-free APIs.
- No broad crypto/netdb/i2np/destination/primitives/transport waiver.
- No direct-clearnet fallback.
- No signing-suite fallback or plaintext/unsecreted/unauthenticated LeaseSet downgrade.
- No blocked durable-inert configuration.
- Secret/private/auth material never becomes diagnostic/log/response-facing merely to simplify integration.
- Rejected protocol payload content is treated as sensitive when it may contain secrets.
- Yosemite remains the exact accepted optional pin unless separately superseded.
- External/upstream activity remains read-only.

## 11. Final-state rule

Full support may be declared only when the mechanically recomputed matrix has zero applicable blockers.

Safe-partial/terminal closure requires every remaining blocker to have an explicit accepted architecture/security disposition, no dependency-ready safe implementation plan to remain, and no unresolved high/medium correctness/security defect. `Terminal` never means a blocked cell is supported.

## 12. Current handoff

**M164 is the sole registered milestone.**

M163 is closed. M165 may be registered only after clean M163+M164 closures. File presence alone never authorizes production work.
