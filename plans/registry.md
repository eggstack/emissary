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
| Post-M154 LeaseSet-security corrective | **active / partial** | `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md` | **no registered successor (M161 closed B; only M162 may be registered next)** |
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

Full Proposal-170 status remains **partial**. M153 remains the current whole-surface runtime/security qualification authority at `336/29/475`.

## Accepted blocked dispositions

- M146: `UseOutproxyPlugin` ×4 remains blocked; no fake provider/direct-clearnet fallback.
- M154/M147/M148: configurable Destination `SigType` ×10 remains blocked; narrow type-7 -> type-11 Red25519 blinding is separate and closed in M156.

## Closed LeaseSet-security ancestry

- M155 — semantic/owner re-freeze; zero production/promotions.
- M156 — neutral Red25519/blinding; zero promotions.
- M157 — modern type-5/no-auth Encrypted LeaseSet2 publication/storage verification/UTC rollover; zero promotions.
- M158 — standard lookup-secret contribution + encrypted-service extended B32; zero promotions.
- M159 — neutral standard PSK client-authorization infrastructure; zero promotions.
- M160 — neutral standard DH (X25519) client-authorization infrastructure; zero promotions.
- M161 — legacy AES/LS1 feasibility gate, outcome B (valid but blocked); zero production/promotions.

M159 closure: `plans/closure/i2pcontrol-proposal-170/159-closure.md` at implementation/closure head `c0bbf9d4`.

M160 closure: `plans/closure/i2pcontrol-proposal-170/160-closure.md` at implementation/closure head recorded therein.

M161 closure: `plans/closure/i2pcontrol-proposal-170/161-closure.md` at implementation/closure head recorded therein (outcome B, zero production).

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

## Remaining chain — reviewed and pre-corrected

| Milestone | Status | Planning correction |
|---|---|---|
| M160 DH/X25519 | closed complete | same four-file/zero-dependency envelope realized; exact M061/M062 authorization observed |
| M161 legacy AES/LS1 gate | closed complete, outcome B | zero-production gate; legacy AES valid but blocked; no successor; M095 unchanged |
| M162 Proposal integration | deferred/unregistered | hard deps satisfied (M160 closed + M161 B); registration pending exact-path amendment |
| M152 final requalification | deferred/unregistered | rebased to closed M156-M160 + M161/M162 outcomes |

M161 closed outcome B (valid legacy-LS1 contract but disproportionate/unsafe; no implementation successor). All five `EncryptLeaseSet` cells remain blocked. M162 hard dependencies (M160 closure + M161 disposition) are satisfied for modern field integration, but M162 is not registered by this closure and still requires an exact-path amendment at registration.

M162 is the first milestone allowed to promote LeaseSet fields. It must materialize the exact ten-mode table and implement persistent I2PControl custody/transactions across all five server families. Neutral primitive availability alone never promotes a field.

## Correct standard property mappings for M162

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
  -> M153 current-head qualification           [CLOSED]
  -> M154 configurable SigType refreeze        [CLOSED; BLOCKED]
  -> M155 LeaseSet semantic refreeze           [CLOSED]
  -> M156 Red25519/blinding                    [CLOSED]
  -> M157 modern Encrypted LS2                 [CLOSED]
  -> M158 lookup-secret/blinded address        [CLOSED]
  -> M159 PSK authorization                    [CLOSED]
  -> M160 DH authorization                     [CLOSED]
  -> M161 legacy AES gate                      [CLOSED; OUTCOME B]
  -> M162 Proposal integration                 [DEFERRED; HARD DEPS SATISFIED]
  -> M152 final qualification                  [DEFERRED]
```

## Canonical containment/registration rules

1. **No milestone is currently registered; only M162 may be registered next** (with exact-path amendment).
2. M161 gate work is closed; M162 registration must revalidate owners and M061/M062 authorization.
3. Proposal/admin policy remains I2PControl-owned; neutral core changes are exact and Proposal-free.
4. No broad core prefix waiver.
5. No secret/auth downgrade or diagnostic exposure.
6. Yosemite remains exact optional authority unless separately superseded.
7. External/upstream access remains read-only.
8. Material path/dependency/architecture deviation requires amendment before implementation.
9. Closure evidence, not parser/serializer reachability, determines support.

Historical closure files remain immutable.