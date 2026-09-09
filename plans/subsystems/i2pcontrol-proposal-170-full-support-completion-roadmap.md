# I2PControl Proposal 170 Full-Support Completion Roadmap

Status: **active / partial; M155/M156 closed, M157 deferred pending amendment**

Pinned Proposal authority:

- I2P Proposal 170 revision `2026-05-20`, status Open.

Current machine authority:

- M095 matrix: `336 apply / 29 blocked_primitive / 475 not_applicable` across 840 TunnelManager option/family cells.

Current execution authority:

- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Current whole-surface qualification authority:

- M153, closed complete at `336/29/475`.

## 1. Purpose

Move the internal fork as far toward exact Proposal-170 support as can be done truthfully while keeping Proposal-specific policy under `emissary-cli/src/i2pcontrol/**` wherever possible and preserving the security-reviewed Emissary core boundary.

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

## 3. Accepted completed/blocked lineage

- M140 corrected residual streaming applicability.
- M141-M145 promoted 11 real runtime-backed cells, reaching `336/29/475`.
- M146 correctly closed `UseOutproxyPlugin` blocked with zero promotions.
- M153 requalified the current production head and is the current whole-surface authority.
- M154 froze the full destination-capable SigType domain and correctly closed the general M147 path blocked; M148 remains blocked behind it.
- M155 closed the LeaseSet-security semantic/owner re-freeze (ten-value table, legacy AES disposition C, narrow blinding frozen) and registered M156.
- M156 closed the neutral Red25519/blinding primitive (spec vectors 1–2, no-std/security review, exact M061/M062 authorization, zero promotions, `336/29/475` unchanged); M157 hard dependency satisfied, amendment pending.

Those closures remain immutable historical evidence.

## 4. Corrected LeaseSet-security line

M154's SigType block does not itself block modern Encrypted LeaseSet2. The current line separates the narrow type-7 Ed25519 -> type-11 Red25519 blinded-key primitive from configurable persistent Destination `SigType`.

Execution order:

```text
M155 LeaseSet semantic/owner refreeze               [CLOSED; ZERO PRODUCTION]
  |
  v
M156 narrow Red25519 + Ed25519 blinding              [CLOSED; ZERO PROMOTION]
  |
  v
M157 modern type-5 Encrypted LeaseSet2 publication   [DEFERRED; ZERO PROMOTION]
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

## 5. Field-completeness rules

- Infrastructure has zero Proposal support value by itself.
- `OptionalLookup` may promote only when its entire valid lookup-secret/blinding contract is operational and interoperable.
- `LeaseSetClientAuths` may promote only when all valid PSK/DH uses are bounded, restart-safe and interoperable.
- `EncryptLeaseSet` may promote only when every valid Proposal enum value is operational for the family. Partial enum-domain support remains blocked.

The legacy `encrypted (aes)` value is intentionally isolated in M155/M161 because it maps to old LS1-style `i2cp.encryptLeaseSet` behavior rather than modern type-5 Encrypted LS2.

## 6. Planning ceilings

From `336/29/475`:

- if modern lookup/auth are fully operational but legacy AES remains a valid unsupported value, a possible safe-partial state is `346/19/475`;
- if all ten `EncryptLeaseSet` values also become operational, a possible state is `351/14/475`;
- full support still additionally requires separate resolution of the 10 `SigType` and 4 `UseOutproxyPlugin` blockers.

These are ceilings, not claims.

## 7. Containment/security rules

Accepted authority remains ADR-0001 through ADR-0005, M061/M062 and M093.

- Proposal/admin/application policy stays under `emissary-cli/src/i2pcontrol/**` wherever possible.
- Neutral core primitives require exact-file M061/M062 authorization and Proposal-free APIs.
- Planning-only M062 entries do not authorize production code or dependencies.
- No broad `crypto/**`, `netdb/**`, `i2np/**`, `destination/**`, `primitives/**` or transport waiver.
- No direct-clearnet fallback for outproxy/plugin work.
- No signing-suite fallback or plaintext/unsecreted/unauthenticated LeaseSet downgrade.
- Secret/private/auth material never becomes response-facing or loggable.
- Yosemite remains the exact accepted optional pin unless separately superseded.
- External/upstream activity remains read-only.

## 8. Final-state rule

M152 may close the workstream as **full** only when the mechanically recomputed matrix has zero applicable blockers.

If one or more accepted architecture/security blockers remain after all safe work is exhausted, M152 may close the current line as **safe partial / terminal under current policy**, but active documentation must remain explicitly partial and enumerate every blocker plus the future decision needed to resume.

## 9. Current handoff

No successor is currently registered. Only M157 may be registered next, with its exact-path amendment and exact M061/M062 authorization. Do not begin M157 implementation until that registration lands.