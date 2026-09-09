# I2PControl Proposal 170 Full-Support Completion Roadmap

Status: **active / partial; M158 registered**

Pinned Proposal authority:

- I2P Proposal 170 revision `2026-05-20`, status Open.

Current machine authority:

- M095 matrix: `336 apply / 29 blocked_primitive / 475 not_applicable` across 840 TunnelManager option/family cells.

Current execution authority:

- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`;
- sole registered implementation handoff: M158.

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
- M157 implemented and closed neutral type-5/no-auth Encrypted LeaseSet2 publication/storage verification/UTC rollover with zero promotions.
- M158 has now been exact-path amended and registered.

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
M158 lookup-secret + blinded-address primitive       [REGISTERED; ZERO PROMOTION]
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

## 5. M158 containment boundary

M158 is a neutral zero-promotion infrastructure milestone using a **strict subset of already-realized M061 exact paths**:

1. `emissary-core/src/crypto/els2.rs`;
2. `emissary-core/src/destination/lease_set.rs`;
3. `emissary-core/src/sam/parser.rs`;
4. `emissary-core/src/sam/session.rs`.

No new M061 source path is introduced. The plan and registry freeze the milestone-specific subset; any production edit outside it requires amendment first.

M062 records:

- zero new production files;
- zero new dependencies;
- zero Cargo/lockfile change;
- zero Yosemite change;
- zero I2PControl production source change.

No broad path permission exists.

## 6. M158 semantic boundary

Direct Java I2P/I2PTunnel source stores the lookup secret in the standard session property as:

```text
i2cp.leaseSetSecret = Base64(UTF8(secret))
```

M158 consumes only that standard representation. The secret is decoded/UTF-8 validated and removed from generic debug-capable SAM options before activation, then carried generation-locally in a dedicated zeroizing/non-`Debug` type.

The existing M156 blinding primitive already accepts secret bytes; M158 threads those bytes through M157 publication and UTC rollover without modifying `red25519.rs` or NetDB.

M158 also implements the canonical current encrypted-service extended B32 form for type-7 -> type-11 blinding: 35 decoded bytes / 56 Base32 characters plus `.b32.i2p`, secret/auth public flags, exact one-byte sigtypes, and Java-compatible CRC-32/XOR header processing.

Core persists no lookup secret. I2PControl Proposal `OptionalLookup` mapping, secret custody, edit/restart transactionality and five-family integration remain M162 work. Therefore M158 has zero Proposal promotions and M095 remains `336/29/475`.

## 7. Reference correction for the modern line

Direct Proposal-170 PR source sets `i2cp.encryptLeaseSet=true` only for legacy AES. Modern blinded/PSK/DH modes are selected by `i2cp.leaseSetType=5`.

M157/M158 therefore operate on the neutral standard type-5 path. M161 retains legacy-AES authority; M162 must use the corrected mapping.

## 8. Field-completeness rules

- Infrastructure has zero Proposal support value by itself.
- `OptionalLookup` promotes only when its entire valid lookup-secret/blinding **administrative and runtime** contract is operational and interoperable.
- `LeaseSetClientAuths` promotes only when all valid PSK/DH uses are bounded, restart-safe and interoperable.
- `EncryptLeaseSet` promotes only when every valid Proposal enum value is operational for the family. Partial enum-domain support remains blocked.

## 9. Planning ceilings

From `336/29/475`:

- modern lookup/auth complete but legacy AES still valid/unsupported: at most `346/19/475`;
- all ten `EncryptLeaseSet` values operational: at most `351/14/475`;
- full support still requires separate resolution of the 10 `SigType` and 4 `UseOutproxyPlugin` blockers.

These are ceilings, not current claims.

## 10. Containment/security rules

Accepted authority remains ADR-0001 through ADR-0005, M061/M062 and M093.

- Proposal/admin/application policy stays under `emissary-cli/src/i2pcontrol/**` wherever possible.
- Neutral core primitives require exact-file authority and Proposal-free APIs.
- A registered successor may use a stricter subset of already-realized exact paths without a redundant source-boundary expansion; that subset must be frozen in the active plan/registry.
- No broad `crypto/**`, `netdb/**`, `i2np/**`, `destination/**`, `primitives/**` or transport waiver.
- No direct-clearnet fallback.
- No signing-suite fallback or plaintext/unsecreted/unauthenticated LeaseSet downgrade.
- Secret/private/auth material never becomes response-facing or loggable.
- Yosemite remains the exact accepted optional pin unless separately superseded.
- External/upstream activity remains read-only.

## 11. Final-state rule

M152 may close the workstream as **full** only when the mechanically recomputed matrix has zero applicable blockers.

If accepted architecture/security blockers remain after all safe work is exhausted, M152 may close the current line as **safe partial / terminal under current policy**, but active documentation must remain explicitly partial and enumerate every remaining blocker plus the future decision needed to resume.

## 12. Current handoff

Execute **M158 only**.

Do not begin M159 until M158 closes and the registry explicitly advances it. Any M158 need outside its exact four-file/zero-dependency budget requires a plan/M061/M062 amendment before the edit.