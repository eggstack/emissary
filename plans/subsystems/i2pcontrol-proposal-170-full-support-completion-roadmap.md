# I2PControl Proposal 170 Full-Support Completion Roadmap

Status: **active / partial; M157 closed, no registered successor**

Pinned Proposal authority:

- I2P Proposal 170 revision `2026-05-20`, status Open.

Current machine authority:

- M095 matrix: `336 apply / 29 blocked_primitive / 475 not_applicable` across 840 TunnelManager option/family cells.

Current execution authority:

- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`;
- registered handoff: none (M157 closed; M158 deferred pending amendment).

Current whole-surface qualification authority:

- M153, closed complete at `336/29/475`.

## 1. Purpose

Move the fork as far toward exact Proposal-170 support as can be done truthfully while keeping Proposal-specific policy under `emissary-cli/src/i2pcontrol/**` wherever possible and preserving exact neutral owners in the security-reviewed core.

Full support means real externally observable behavior. Parser acceptance, persistence, serializer reachability, dormant fields, defaults, aliases or approximate semantics do not count.

## 2. Current residual inventory

| Cluster | Cells | Current disposition |
|---|---:|---|
| `SigType` | 10 | blocked by M154/M147 general configurable-destination path |
| `EncryptLeaseSet` | 5 | blocked; corrected LeaseSet-security line active |
| `OptionalLookup` | 5 | blocked; corrected LeaseSet-security line active |
| `LeaseSetClientAuths` | 5 | blocked; corrected LeaseSet-security line active |
| `UseOutproxyPlugin` | 4 | blocked by M146 provider/security disposition |
| **Total** | **29** | |

## 3. Accepted lineage

- M140 corrected residual streaming applicability.
- M141-M145 promoted 11 real runtime-backed cells, reaching `336/29/475`.
- M146 closed `UseOutproxyPlugin` blocked with zero promotions.
- M153 requalified the current production head and remains the whole-surface authority.
- M154 closed the general configurable SigType path blocked; M147/M148 remain blocked.
- M155 re-froze the LeaseSet-security line and separated modern ELS2 from configurable Destination SigType.
- M156 implemented the narrow neutral Red25519/blinding primitive with zero promotions.
- M157 has now been implemented and closed with its exact ten-file budget reconciled into M061/M062.

Historical closures remain immutable.

## 4. Corrected LeaseSet-security line

```text
M155 LeaseSet semantic/owner refreeze               [CLOSED; ZERO PRODUCTION]
  |
  v
M156 narrow Red25519 + Ed25519 blinding              [CLOSED; ZERO PROMOTION]
  |
  v
M157 modern type-5 Encrypted LeaseSet2 publication   [CLOSED; ZERO PROMOTION]
  |
  v
M158 lookup-secret + blinded-address primitive       [DEFERRED]
  |
  v
M159 PSK client-authorization primitive              [DEFERRED; ZERO PROMOTION]
  |
  v
M160 DH/X25519 client-authorization primitive        [DEFERRED; ZERO PROMOTION]
  |
  +--> M161 legacy AES/LS1 feasibility               [DEFERRED; ZERO PRODUCTION]
  |
  v
M162 Proposal LeaseSet-field integration             [DEFERRED; CONDITIONAL PROMOTION]
  |
  v
M152 final whole-surface requalification             [DEFERRED; ZERO PROMOTION]
```

M149-M151 are superseded historical drafts and must not be executed directly.

## 5. M157 containment boundary

M157 is a neutral zero-promotion infrastructure milestone. It had an exact ten-file core budget, now realized and reconciled into M061/M062 ordinary ledgers. No broad path permission exists.

It authorizes no:

- Cargo/dependency/lockfile change;
- Yosemite change;
- `emissary-cli/src/i2pcontrol/**` production change;
- transport/tunnel/router/frontend change;
- generic signature-suite migration.

The first production commit must reconcile any newly realized source path from M061's `registered_pending` ledger into the ordinary exact upstream-diff allowlist atomically with the code change.

## 6. Reference correction for the modern line

Direct Proposal-170 PR source sets `i2cp.encryptLeaseSet=true` only for legacy AES. Modern blinded/PSK/DH modes are selected by `i2cp.leaseSetType=5`.

M157 therefore implements the neutral standard type-5/no-auth publication primitive and does not alias the legacy AES flag to modern ELS2. M161 retains legacy-AES authority; M162 must use the corrected mapping.

## 7. Field-completeness rules

- Infrastructure has zero Proposal support value by itself.
- `OptionalLookup` promotes only when its entire valid lookup-secret/blinding contract is operational and interoperable.
- `LeaseSetClientAuths` promotes only when all valid PSK/DH uses are bounded, restart-safe and interoperable.
- `EncryptLeaseSet` promotes only when every valid Proposal enum value is operational for the family. Partial enum-domain support remains blocked.

## 8. Planning ceilings

From `336/29/475`:

- modern lookup/auth complete but legacy AES still valid/unsupported: at most `346/19/475`;
- all ten `EncryptLeaseSet` values operational: at most `351/14/475`;
- full support still requires separate resolution of the 10 `SigType` and 4 `UseOutproxyPlugin` blockers.

These are ceilings, not current claims.

## 9. Containment/security rules

Accepted authority remains ADR-0001 through ADR-0005, M061/M062 and M093.

- Proposal/admin/application policy stays under `emissary-cli/src/i2pcontrol/**` wherever possible.
- Neutral core primitives require exact-file authority and Proposal-free APIs.
- No broad `crypto/**`, `netdb/**`, `i2np/**`, `destination/**`, `primitives/**` or transport waiver.
- No direct-clearnet fallback.
- No signing-suite fallback or plaintext/unsecreted/unauthenticated LeaseSet downgrade.
- Secret/private/auth material never becomes response-facing or loggable.
- Yosemite remains the exact accepted optional pin unless separately superseded.
- External/upstream activity remains read-only.

## 10. Final-state rule

M152 may close the workstream as **full** only when the mechanically recomputed matrix has zero applicable blockers.

If accepted architecture/security blockers remain after all safe work is exhausted, M152 may close the current line as **safe partial / terminal under current policy**, but active documentation must remain explicitly partial and enumerate every remaining blocker plus the future decision needed to resume.

## 11. Current handoff

M157 is closed. Only M158 may be registered next, with exact M061/M062 authorization in that registration commit.

Do not begin M158 until the registry explicitly advances it.