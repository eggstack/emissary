# Emissary Active Planning Registry

This file is the compact control surface for active planning.

Canonical direction:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`

Accepted Proposal-170 architecture/security authority:

- ADR-0001 through ADR-0005;
- M061/M062 containment;
- M093 tunnel security.

Pinned Proposal 170 revision: `2026-05-20` (Open).

Authorized internal repositories:

- `eggstack/emissary`;
- `eggstack/yosemite` only under ADR-0005 and Yosemite's own registered plans.

All upstream/third-party repositories and maintainer channels remain read-only.

## Active roadmaps

| Subsystem | Status | Roadmap | Current handoff |
|---|---|---|---|
| Proposal 170 full-support completion | **active / partial** | `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` | subordinate to current corrective roadmap |
| Post-M152 blocked-state/security corrective | **active** | `plans/subsystems/i2pcontrol-proposal-170-post-m152-blocked-state-corrective-roadmap.md` | **M165 registered / dependency-ready** |
| Post-M154 LeaseSet-security corrective | historical through M152 | `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md` | M152 historical safe-partial closure; superseded as active handoff by M163 |
| Post-M146 corrective | historical through M154 | `plans/subsystems/i2pcontrol-proposal-170-post-m146-corrective-roadmap.md` | M153 complete; M154 disposition C |
| Residual primitive completion | historical through M146 | `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md` | M140-M145 complete; M146 blocked |
| Session-lifecycle completion | closed complete | `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md` | M134 complete |
| I2PControl containment | accepted authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | M061/M062 regression authority |

## Current support state

M095 remains:

- `336 apply`;
- `29 blocked_primitive`;
- `475 not_applicable`;
- `840` total cells.

Residual blockers:

- `SigType` — 10;
- `EncryptLeaseSet` — 5;
- `OptionalLookup` — 5;
- `LeaseSetClientAuths` — 5;
- `UseOutproxyPlugin` — 4.

Full Proposal-170 status remains **partial**. M153 remains whole-surface runtime/security qualification ancestry at `336/29/475`. M152 closed as safe partial / terminal on the M162 head, but post-closure review identified two corrective defects: blocked LeaseSet-security values can be durably persisted (including retained generation history), and malformed SAM commands can log raw secret-bearing input. M095 also retains stale production-head metadata. M152 remains immutable historical evidence; M163-M165 are the accepted corrective chain.

Historical qualification/promotion lineage retained for guard agreement: M130 historical post-corrective requalification (M127 token lifetime, M128 batch conformance, M129 fail-closed TLS); M139 historical whole-surface requalification (superseded by M153); M141 `UniqueLocalAddressPerClient` ×2 (327-era promotion); M142 `SSLProxies`/`JumpList` ×2 (329-era promotion); M143 retained `Profile:client`.

## Accepted blocked dispositions

- M146: `UseOutproxyPlugin` ×4 remains blocked; no fake provider/direct-clearnet fallback.
- M154/M147/M148: configurable Destination `SigType` ×10 remains blocked; narrow type-7 -> type-11 Red25519 blinding is separate and closed in M156.
- M161: `EncryptLeaseSet="encrypted (aes)"` is a valid legacy-LS1 contract but remains blocked; no LS1 resurrection is authorized.
- M162: all fifteen LeaseSet-security Proposal cells remain blocked; modern neutral primitives do not count as Proposal support without the complete integration contract.

## Closed LeaseSet-security ancestry

- M155 — semantic/owner re-freeze; zero production/promotions.
- M156 — neutral Red25519/blinding; zero promotions.
- M157 — modern type-5/no-auth Encrypted LeaseSet2 publication/storage verification/UTC rollover; zero promotions.
- M158 — standard lookup-secret contribution + encrypted-service extended B32; zero promotions.
- M159 — neutral standard PSK client-authorization infrastructure; zero promotions.
- M160 — neutral standard DH (X25519) client-authorization infrastructure; zero promotions.
- M161 — legacy AES/LS1 feasibility gate, outcome B (valid but blocked); zero production/promotions.
- M162 — Proposal LeaseSet-security blocked integration (typed/redacted domain, ten-mode table, five-family fail-closed runtime gates); zero promotions.
- M152 — historical final whole-surface requalification on the M162 head; safe partial / terminal at closure, superseded as active authority by the post-M152 corrective chain.

Historical promotion lineage: M141 `UniqueLocalAddressPerClient` ×2 applied; M142 `SSLProxies`/`JumpList` ×2 applied; M143 retained `Profile:client` applied; M144 `UseSSL` ×4 applied; M145 `MultiHoming` ×2 applied.

M159 closure: `plans/closure/i2pcontrol-proposal-170/159-closure.md` at implementation/closure head `c0bbf9d4`.

M160 closure: `plans/closure/i2pcontrol-proposal-170/160-closure.md` at implementation/closure head recorded therein.

M161 closure: `plans/closure/i2pcontrol-proposal-170/161-closure.md` at implementation/closure head recorded therein (outcome B, zero production).

M162 closure: `plans/closure/i2pcontrol-proposal-170/162-closure.md` at implementation/closure head recorded therein (blocked integration, zero promotions, nine-file I2PControl subset).

## Closed handoff — M163

Plan:

- `plans/implementation/i2pcontrol-proposal-170/163-blocked-leaseset-state-persistence-corrective.md`.

Status: **closed as complete** (`plans/closure/i2pcontrol-proposal-170/163-closure.md`).

Class: I2PControl-only correctness/security corrective.

Promotion budget: **zero Proposal cells** (M095 remains `336/29/475`).

### Exact production budget

Only:

1. `emissary-cli/src/i2pcontrol/tunnel_manager.rs`;
2. `emissary-cli/src/i2pcontrol/stores/tunnel_store.rs`;
3. `emissary-cli/src/i2pcontrol/stores/generation_store.rs`.

All three are inside the existing I2PControl policy root. No new M061 non-policy path waiver exists. M062 records the exact three-path current registration with zero dependency/manifest/lockfile/Yosemite/core expansion.

### Corrective contract

M162 correctly blocked LeaseSet-security at backend/session runtime allocation, but its typed CRUD layer can durably store blocked `EncryptLeaseSet`, `OptionalLookup`, and `LeaseSetClientAuths` fields. `OptionalLookup` and per-user keys are secret-bearing serde state.

A single sanitized publish is not sufficient because `GenerationStore` retains five prior generations and ordinary cleanup is best-effort. M163 therefore requires a crash-resumable `legacy -> sanitized_pending_history_purge -> scrub_complete` migration: clear all blocked fields, establish at least two newest clean fallback generations, fail-closed purge all older contaminated generation files with directory sync, then publish the completed marker. Interrupted pending scrubs must resume before StartOnLoad.

Create/Edit with any supplied blocked LeaseSet-security field must also fail before any `TunnelStore` mutation. Existing backend/session gates remain defense in depth.

No Proposal promotion/demotion occurs. A fourth production path, new dependency, Yosemite/core change, new secret store, or matrix change requires amendment before implementation.

## Closed handoff — M164

Plan:

- `plans/implementation/i2pcontrol-proposal-170/164-sam-invalid-command-secret-redaction-corrective.md`.

Status: **closed as complete** (`plans/closure/i2pcontrol-proposal-170/164-closure.md`); M163 closure satisfied.

Class: neutral SAM security hardening.

Exact production budget: `emissary-core/src/sam/socket.rs` only. That path is already an exact M061 neutral owner.

M164 removes the raw rejected command field from rejected-SAM logging. The complete rejected payload is treated as sensitive; only structural metadata such as observation id, peer and byte length is logged. No parser/session semantics, dependency, Yosemite or Proposal disposition changed.

## Active handoff — M165

Plan:

- `plans/implementation/i2pcontrol-proposal-170/165-post-corrective-current-head-requalification.md`.

Status: **registered / dependency-ready; hard-depends on M163 + M164 closure**.

M165 is zero-production/zero-promotion. It refreshes M095 `current_production_head` to the actual last production-bearing M164 closure commit, mechanically recomputes `336/29/475`, re-runs whole-surface behavioral/security/containment qualification through M164, and becomes the new current-head safe-partial authority only if no high/medium defect remains.

M165 is the sole registered implementation handoff after clean M163+M164 closures. It is zero-production and zero-promotion; it must refresh M095 only after current-head qualification succeeds.

## Closed handoff — M160 (realized record)

Plan:

- `plans/implementation/i2pcontrol-proposal-170/160-leaseset-dh-client-authorization-primitive.md`.

Status: **closed as complete** (`plans/closure/i2pcontrol-proposal-170/160-closure.md`).

Class: neutral standard DH (X25519) client-authorization infrastructure.

Promotion budget: **zero Proposal cells** (observed; M095 remains `336/29/475`).

### Exact production budget (realized)

M160 modified exactly:

1. `emissary-core/src/crypto/els2.rs`;
2. `emissary-core/src/destination/lease_set.rs`;
3. `emissary-core/src/sam/parser.rs`;
4. `emissary-core/src/sam/session.rs`.

These four files are already exact realized M061 owners from M157/M158/M159. M160 creates no new M061 path waiver. No other production path was authorized or changed.

M062 records the realized M160 dependency budget:

- no new file;
- no new direct dependency;
- no Cargo manifest change;
- no lockfile change;
- no Yosemite change;
- no I2PControl production-source change.

### Realized DH reference contract (M160 closed)

Standard properties (realized):

```text
i2cp.leaseSetType=5
i2cp.leaseSetAuthType=1
i2cp.leaseSetPrivKey=Base64(32B X25519 private)
i2cp.leaseSetClient.dh.N=[Base64(UTF8(name)) ":"] Base64(32B X25519 public)
```

The public key derived from the base private key is always in the authorized set. Indexed entries are additional and are only required by Proposal per-user modes, not by the neutral non-per-user DH runtime. Duplicate entries are preserved exactly as configured.

Layer-1 DH uses flags `0x01`, a fresh ephemeral X25519 keypair + fresh 32-byte auth cookie per regenerated object, `ELS2_XCA` HKDF-SHA256 52-byte derivation over `shared || cpk || subcredential || published_BE` salted by the ephemeral public key, 8-byte client IDs + 32-byte encrypted auth cookies, explicit all-zero shared-secret rejection, and auth-cookie-bound L2 derivation. Multi-client order is randomized.

M160 adopts the pinned Java `EncryptedLeaseSet.MAX_ENCRYPTED_SIZE=4096` encrypted-data ceiling as the allocation/O(N) work bound. Checked complete-size calculation occurs before any per-client X25519 work. Clients are never silently dropped to fit.

DH keys are extracted from generic SAM options into dedicated zeroizing/non-`Debug` state before activation. Core persists no DH material. Proposal persistence/edit/restart remains M162.

### Hard stops (none triggered)

M160 closed without requiring:

- a fifth production file;
- any dependency/Cargo/lock/Yosemite change;
- I2PControl production changes;
- another NetDB/client resolver or publication owner;
- debug-capable DH storage;
- >4096 authenticated encrypted data;
- plaintext/no-auth downgrade.

M095 remains `336/29/475` after M160.

## Closed handoff — M159 (realized record)

Plan:

- `plans/implementation/i2pcontrol-proposal-170/159-leaseset-psk-client-authorization-primitive.md`.

Status: **closed as complete** (`plans/closure/i2pcontrol-proposal-170/159-closure.md`).

Class: neutral standard PSK client-authorization infrastructure.

Promotion budget: **zero Proposal cells** (observed; M095 remains `336/29/475`).

### Exact production budget (realized)

M159 modified exactly:

1. `emissary-core/src/crypto/els2.rs`;
2. `emissary-core/src/destination/lease_set.rs`;
3. `emissary-core/src/sam/parser.rs`;
4. `emissary-core/src/sam/session.rs`.

These four files are already exact realized M061 owners from M157/M158. M159 creates no new M061 path waiver. No other production path was authorized or changed.

M062 records the realized M159 dependency budget:

- no new file;
- no new direct dependency;
- no Cargo manifest change;
- no lockfile change;
- no Yosemite change;
- no I2PControl production-source change.

### Realized PSK reference contract (M159 closed)

Standard properties (realized):

```text
i2cp.leaseSetType=5
i2cp.leaseSetAuthType=2
i2cp.leaseSetPrivKey=Base64(32B)
i2cp.leaseSetClient.psk.N=[Base64(UTF8(name)) ":"] Base64(32B)
```

The base `leaseSetPrivKey` is itself an authorized PSK. Indexed entries are additional and are only required by Proposal per-user modes, not by the neutral non-per-user PSK runtime. Duplicate entries are preserved exactly as configured.

Layer-1 PSK uses flags `0x03`, fresh 32-byte auth cookie + auth salt, `ELS2PSKA` HKDF-SHA256 52-byte derivation, 8-byte client IDs + 32-byte encrypted auth cookies, and auth-cookie-bound L2 derivation. Multi-client order is randomized.

M159 adopted the pinned Java `EncryptedLeaseSet.MAX_ENCRYPTED_SIZE=4096` encrypted-data ceiling as the allocation/O(N) work bound. Clients are never silently dropped to fit.

PSK keys are extracted from generic SAM options into dedicated zeroizing/non-`Debug` state before activation. Core persists no PSK material. Proposal persistence/edit/restart remains M162.

### Hard stops (none triggered)

M159 closed without requiring:

- a fifth production file;
- any dependency/Cargo/lock/Yosemite change;
- I2PControl production changes;
- another NetDB/client resolver or publication owner;
- debug-capable PSK storage;
- >4096 authenticated encrypted data;
- plaintext/no-auth downgrade.

M095 remains `336/29/475` after M159.

## Remaining chain — reviewed and corrected

| Milestone | Status | Planning correction |
|---|---|---|
| M160 DH/X25519 | closed complete | same four-file/zero-dependency envelope realized; exact M061/M062 authorization observed |
| M161 legacy AES/LS1 gate | closed complete, outcome B | zero-production gate; legacy AES valid but blocked; no successor; M095 unchanged |
| M162 Proposal integration | closed complete, blocked integration | nine-file I2PControl subset realized; typed domain + ten-mode table + runtime fail-closed gates; zero promotions; post-M152 review found durable blocked-state defect |
| M152 final requalification | closed historical safe partial | immutable evidence for its reviewed head, superseded as active authority by the M163-M165 corrective chain |
| M163 blocked-state persistence/history | closed complete | exact three-file I2PControl corrective; zero promotions |
| M164 SAM rejected-command logging | **closed complete** | exact one-file neutral core hardening; zero promotions |
| M165 post-corrective requalification | **registered / dependency-ready** | zero-production authority refresh; hard-depends M163+M164 |

M161 closed outcome B (valid legacy-LS1 contract but disproportionate/unsafe; no implementation successor). All five `EncryptLeaseSet` cells remain blocked. M162 correctly kept modern field integration blocked by the frozen Yosemite base-key/duplicate/bound gaps plus inherited M161-B legacy block, but its CRUD persistence behavior requires M163 correction. No partial-enum apply is claimed; M095 remains unchanged.

## Correct standard property mappings for M162 lineage

```text
legacy AES:
  i2cp.encryptLeaseSet=true

modern:
  i2cp.leaseSetType=5

OptionalLookup:
  i2cp.leaseSetSecret=Base64(UTF8(value))

PSK:
  i2cp.leaseSetAuthType=2
  i2cp.leaseSetPrivKey=Base64(32B)
  i2cp.leaseSetClient.psk.N=Base64(UTF8(Name)) + ":" + Key

DH:
  i2cp.leaseSetAuthType=1
  i2cp.leaseSetPrivKey=Base64(32B X25519 private)
  i2cp.leaseSetClient.dh.N=Base64(UTF8(Name)) + ":" + Key
```

Legacy `i2cp.encryptLeaseSet=true` MUST NOT be used as the selector for modern type 5.

## Promotion rules

- Neutral infrastructure has zero support value by itself.
- `OptionalLookup` promotes only after full Proposal mapping/persistence/edit/restart/redaction/interoperability across the five server families.
- `LeaseSetClientAuths` promotes only after the full PSK/DH list contract is bounded, persistent, restart-safe and interoperable.
- `EncryptLeaseSet` promotes only if every valid enum value is operational for the family. Legacy AES remaining valid/unsupported keeps all five cells blocked.

Planning ceilings from `336/29/475`:

- OptionalLookup + LeaseSetClientAuths complete, EncryptLeaseSet blocked: at most `346/19/475`;
- all ten EncryptLeaseSet values complete: at most `351/14/475`.

These are ceilings, not current claims.

## Current execution chain

```text
M146 UseOutproxyPlugin                         [CLOSED BLOCKED]
  -> M153 current-head qualification           [CLOSED HISTORICAL]
  -> M154 configurable SigType refreeze        [CLOSED; BLOCKED]
  -> M155 LeaseSet semantic refreeze           [CLOSED]
  -> M156 Red25519/blinding                    [CLOSED]
  -> M157 modern Encrypted LS2                 [CLOSED]
  -> M158 lookup-secret/blinded address        [CLOSED]
  -> M159 PSK authorization                    [CLOSED]
  -> M160 DH authorization                     [CLOSED]
  -> M161 legacy AES gate                      [CLOSED; OUTCOME B]
  -> M162 Proposal integration                 [CLOSED; BLOCKED INTEGRATION]
  -> M152 final qualification                  [CLOSED HISTORICAL; SAFE PARTIAL]
  -> M163 blocked-state persistence/history    [CLOSED]
  -> M164 SAM invalid-command secret redaction [CLOSED]
  -> M165 current-head requalification         [REGISTERED]
```

## Canonical containment/registration rules

1. **M165 is the sole registered Proposal-170 implementation handoff.**
2. M164 was registered only after a clean M163 closure; M165 is registered only after clean M163+M164 closures.
3. Proposal/admin policy remains I2PControl-owned; neutral core changes are exact and Proposal-free.
4. No broad core prefix waiver.
5. No secret/auth downgrade or diagnostic exposure.
6. Yosemite remains exact optional authority unless separately superseded.
7. External/upstream access remains read-only.
8. Material path/dependency/architecture deviation requires amendment before implementation.
9. Closure evidence, not parser/serializer/persistence reachability, determines support.
10. A blocked capability must fail before runtime allocation and before durable effect; inert persisted configuration is not accepted behavior.
11. Rejected SAM payload content is treated as sensitive and must not be logged verbatim.

Historical closure files remain immutable.

## Historical status rules

- **M149-M151 remain superseded/unregistered. Do not execute or revive them.** Their useful semantics were replaced by M155-M162.
- **M147 remains closed blocked and M148 remains blocked/deferred behind M147 under M154 disposition C.** M163-M165 do not reopen configurable SigType.
- **M146 remains closed blocked** for `UseOutproxyPlugin`; this line does not reopen it.
- Neutral M156-M160 infrastructure alone never promotes Proposal fields.
