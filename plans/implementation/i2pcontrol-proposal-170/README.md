# Proposal 170 Implementation Handoffs

Status: **partial Proposal 170 support; M152 retained as historical safe-partial qualification; M163 and M164 are closed, M165 is the sole registered corrective handoff**.

Pinned Proposal revision: `2026-05-20` (Open).

Current M095 state: `336 apply / 29 blocked_primitive / 475 not_applicable` across 840 cells.

Residual blockers: 10 `SigType`, 5 `EncryptLeaseSet`, 5 `OptionalLookup`, 5 `LeaseSetClientAuths`, 4 `UseOutproxyPlugin`.

Historical qualification/promotion lineage: M139 historical whole-surface requalification (superseded by M153); M141 `UniqueLocalAddressPerClient` ×2; M142 `SSLProxies`/`JumpList` ×2; M143 retained `Profile:client`; M144 `UseSSL` ×4; M145 `MultiHoming` ×2; M153 whole-surface ancestry at `336/29/475`; M152 historical safe-partial requalification on the M162 head.

## Current execution authority

Roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m152-blocked-state-corrective-roadmap.md`.

Closed corrective:

- **M163** `163-blocked-leaseset-state-persistence-corrective.md` — I2PControl-only correctness/security corrective; closed with zero Proposal promotions.

Closed corrective:

- **M164** `164-sam-invalid-command-secret-redaction-corrective.md` — neutral SAM log hardening after M163; closed with zero Proposal promotions.

Registered handoff:

- **M165** `165-post-corrective-current-head-requalification.md` — zero-production current-head authority refresh after M163+M164; dependency-ready.

M149-M151 remain superseded historical drafts and must not be executed. M147/M148 remain blocked under M154-C. M146 remains blocked for `UseOutproxyPlugin`.

## Why M163 exists

M162 correctly left all fifteen LeaseSet-security Proposal cells blocked and added backend/session fail-before-allocation gates. But its typed CRUD layer still permits blocked `EncryptLeaseSet`, `OptionalLookup`, and `LeaseSetClientAuths` values to enter `TunnelDefinition` and be durably committed by `TunnelStore` before Start later rejects them.

`OptionalLookup` and `LeaseSetClientAuths` are secret-bearing serde state. That is not acceptable merely because Debug/Get are redacted: blocked Proposal values must fail before allocation **and before durable effect**.

A second issue exists for already-written state. `GenerationStore` retains the current generation plus up to five prior generations and ordinary cleanup is best-effort. Therefore a single sanitized generation would leave older M162-era secret-bearing files on disk.

## M163 exact production budget

Only:

1. `emissary-cli/src/i2pcontrol/tunnel_manager.rs`
2. `emissary-cli/src/i2pcontrol/stores/tunnel_store.rs`
3. `emissary-cli/src/i2pcontrol/stores/generation_store.rs`

All are inside the existing I2PControl policy root. No non-I2PControl M061 exception is added.

M163 authorizes no core/Yosemite/dependency/Cargo/lockfile/server-secret/backend/session-runtime change and no matrix promotion/demotion.

### M163 required behavior

Create/Edit containing any supplied blocked LeaseSet-security field must fail before `TunnelStore` mutation. Existing M162 backend/session rejection remains defense in depth.

Historical scrub must be crash-resumable:

```text
legacy/unstarted
  -> sanitized_pending_history_purge
  -> scrub_complete
```

Required sequence:

1. load newest valid generation using existing corruption fallback;
2. clear all three blocked LeaseSet-security fields from every affected control-plane definition;
3. publish a clean pending generation;
4. publish a second clean pending generation so at least two clean fallback files exist;
5. invoke a narrow fail-closed generation-history purge retaining only the newest two clean generations, with confined/non-symlink path checks and directory sync;
6. publish `scrub_complete` only after successful purge;
7. interruption/failure before completion must prevent StartOnLoad and resume on next load.

Do not change ordinary `GenerationStore::cleanup()` retention semantics globally.

## M164 completed SAM log hardening

Current `emissary-core/src/sam/socket.rs` logs the complete rejected UTF-8 `%command` when `SamCommand::parse()` returns `None`. Malformed standard SAM/I2CP commands can therefore echo lookup passwords or PSK/DH key material into logs.

M164 is intentionally one production file only:

- `emissary-core/src/sam/socket.rs`

It must remove rejected payload content wholesale and log only safe structural metadata such as observation id, peer and command byte length. No ad-hoc secret-key regex/list; no parser/session/connection semantic change; no dependency/Yosemite/I2PControl production change; zero Proposal promotions.

M164 is closed by `plans/closure/i2pcontrol-proposal-170/164-closure.md`.

## M165 registered current-head requalification

After clean M163+M164 closures, M165 is zero-production/zero-promotion. It must:

- update M095 `current_production_head` to the actual last production-bearing M164 closure commit;
- mechanically recompute all 840 cells and the exact 29 residual blockers;
- re-run behavioral/security/containment qualification through M164;
- specifically prove no blocked LeaseSet durable effect/history residue and no malformed-SAM secret logging;
- become the new safe-partial current-head authority only if no high/medium defect remains.

## Closed LeaseSet-security lineage

- M155 — semantic/owner re-freeze; zero production/promotions.
- M156 — Red25519/blinding; zero promotions.
- M157 — modern type-5/no-auth Encrypted LeaseSet2 publication/storage verification/UTC rollover; zero promotions.
- M158 — lookup-secret contribution + encrypted-service extended B32; zero promotions.
- M159 — neutral PSK client authorization; zero promotions.
- M160 — neutral DH/X25519 client authorization; zero promotions.
- M161 — legacy AES/LS1 feasibility outcome B; valid but blocked, zero production/promotions.
- M162 — Proposal LeaseSet-security typed/redacted blocked integration; zero promotions.
- M152 — historical safe-partial requalification on the M162 head.

## M160 realized record

M160 modified exactly:

1. `emissary-core/src/crypto/els2.rs`
2. `emissary-core/src/destination/lease_set.rs`
3. `emissary-core/src/sam/parser.rs`
4. `emissary-core/src/sam/session.rs`

All four already exist in M061's realized exact allowlist. M160 created no new M061 path waiver and no new dependency/Cargo/lock/Yosemite/I2PControl production change.

Standard DH properties:

```text
i2cp.leaseSetType=5
i2cp.leaseSetAuthType=1
i2cp.leaseSetPrivKey=Base64(32B X25519 private)
i2cp.leaseSetClient.dh.N=[Base64(UTF8(name)) ":"] Base64(32B X25519 public)
```

The base private key's derived public is always in the authorized set; indexed entries are additional; duplicate entries are preserved. M160 uses fresh ephemeral X25519 + auth cookie per regenerated object, `ELS2_XCA` 52-byte HKDF-SHA256 derivation, explicit all-zero shared-secret rejection, randomized client records and the pinned 4096-byte encrypted-data work bound. Core persists no DH material.

M159 implements the analogous PSK layer with `authType=2`, `ELS2PSKA`, duplicate preservation, the same 4096-byte bound and zero Proposal promotions.

## Correct standard property mappings retained from M162

```text
legacy AES -> i2cp.encryptLeaseSet=true
modern -> i2cp.leaseSetType=5
OptionalLookup -> i2cp.leaseSetSecret=Base64(UTF8(value))
PSK -> authType=2 + persistent base key + optional indexed psk clients
DH  -> authType=1 + persistent X25519 private base key + optional indexed dh clients
```

Legacy `i2cp.encryptLeaseSet=true` must never be used as the selector for modern type 5.

## Current execution chain

```text
M159 PSK authorization                       [CLOSED]
  -> M160 DH/X25519 authorization            [CLOSED]
  -> M161 legacy AES/LS1 gate                [CLOSED; OUTCOME B]
  -> M162 Proposal integration               [CLOSED; BLOCKED INTEGRATION]
  -> M152 historical requalification         [CLOSED; SAFE PARTIAL]
  -> M163 blocked-state persistence/history  [CLOSED]
  -> M164 SAM invalid-command redaction      [REGISTERED]
  -> M165 current-head requalification       [DEFERRED]
```

## Containment rules

- Proposal/admin/application policy stays under `emissary-cli/src/i2pcontrol/**` wherever possible.
- Neutral core seams require exact-file authority and Proposal-free APIs.
- No broad crypto/netdb/i2np/destination/primitives/transport waiver.
- No plaintext/unsecreted/unauthenticated downgrade.
- No blocked durable-inert configuration.
- No secret/private/auth material in generic diagnostics, logs or response-facing storage.
- External/upstream access remains read-only.

File presence alone never authorizes production work; only the sole registered handoff does.
