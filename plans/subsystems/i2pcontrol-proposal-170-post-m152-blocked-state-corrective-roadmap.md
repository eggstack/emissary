# Proposal 170 Post-M152 Blocked-State Corrective Roadmap

Status: **active; M163 and M164 closed, M165 registered / dependency-ready**

Authority baseline: M152 closure on `11a48c1b87fadb6bce9d120f41025eaec563e837` plus post-closure current-code review.

## Goal

Repair the remaining correctness/security defects discovered after M152 without reopening completed neutral LeaseSet cryptography or accepted blocked dispositions.

The current matrix remains `336 apply / 29 blocked_primitive / 475 not_applicable`. This roadmap does not seek additional Proposal support. It restores two invariants before current-head requalification:

1. blocked capabilities produce neither runtime allocation nor durable inert state, including retained historical generations;
2. malformed SAM commands cannot echo secret-bearing input into logs.

## Findings

### A. Blocked LeaseSet state is durably accepted

M162 correctly left all fifteen LeaseSet-security Proposal cells blocked and added backend/session fail-before-allocation gates. However, its typed CRUD layer permits blocked `EncryptLeaseSet`, `OptionalLookup`, and `LeaseSetClientAuths` values to be serialized inside `TunnelDefinition` and committed by the generic `TunnelStore` on Create/Edit before lifecycle validation rejects them.

`OptionalLookup` and `LeaseSetClientAuths` are secret-bearing. Redacted Debug/Get behavior is not sufficient: unsupported state is still durably accepted, and M162 deliberately did not allocate the dedicated LeaseSet secret store because full integration remained blocked.

The first M163 draft assumed one sanitized publication was enough. Current `GenerationStore` behavior proves otherwise: ordinary cleanup retains five prior generations and is best-effort. A correct historical scrub therefore needs an exact generation-store purge primitive plus a crash-resumable marker/state machine so contaminated generations cannot survive a nominally successful migration.

### B. Rejected SAM commands can log secrets

`emissary-core/src/sam/socket.rs` currently emits the complete `%command` string when `SamCommand::parse()` rejects UTF-8 input. M158-M160 standard SAM/I2CP options may carry lookup passwords, PSK keys, X25519 private keys and per-client keys. Valid inputs are removed from generic debug-capable state, but malformed inputs can still be echoed to logs.

This is a neutral SAM logging defect. It is independent of Proposal field support and should be fixed in the exact canonical socket owner before final qualification.

### C. M095 production-head metadata is stale

M095 still carries an older `current_production_head` even though later production-bearing M159/M160/M162 commits exist. After M163 and M164, a zero-production M165 must refresh that metadata to the actual last production-bearing corrective head and requalify the full surface.

## Sequence

```text
M152 historical safe-partial qualification
  -> M163 blocked LeaseSet durable/history corrective   [CLOSED]
  -> M164 SAM invalid-command secret redaction          [CLOSED]
  -> M165 post-corrective current-head requalification  [REGISTERED / DEPENDENCY-READY]
```

### M163 — blocked LeaseSet durable/history corrective

Class: I2PControl-only correctness/security corrective.

Exact production owners:

- `emissary-cli/src/i2pcontrol/tunnel_manager.rs`
- `emissary-cli/src/i2pcontrol/stores/tunnel_store.rs`
- `emissary-cli/src/i2pcontrol/stores/generation_store.rs`

Responsibilities:

- reject supplied blocked LeaseSet-security fields on Create/Edit before durable mutation;
- retain existing backend/session rejection as defense in depth;
- sanitize M162-era typed state on load;
- publish at least two clean fallback generations before removing contaminated history;
- use an internal crash-resumable pending/completed scrub marker;
- fail closed on contaminated-history deletion/directory-sync failure and resume next load;
- preserve unrelated definition state and ordinary generation semantics;
- zero matrix promotions/demotions;
- no core, Yosemite, dependency or server-secret-store changes.

### M164 — SAM invalid-command secret redaction

Class: neutral core security hardening.

Exact production owner:

- `emissary-core/src/sam/socket.rs`

Responsibilities:

- remove raw rejected command content from tracing;
- log only safe structural metadata such as observation id, peer and byte length;
- treat the entire rejected command as sensitive rather than maintaining an ad-hoc secret-key redaction list;
- preserve parser/session/connection behavior;
- zero dependencies and zero Proposal promotions.

M164 closed after clean M163 closure. `sam/socket.rs` is already an exact realized M061 neutral owner, so no broad source-boundary waiver was needed. Closure evidence is recorded in `plans/closure/i2pcontrol-proposal-170/164-closure.md`.

### M165 — post-corrective qualification

Class: zero-production requalification.

Responsibilities:

- set M095 `current_production_head` to the actual last production-bearing M164 closure commit;
- mechanically re-evaluate all 840 cells and the exact 29 residual blockers;
- rerun behavioral/security/containment evidence across M127-M164;
- specifically prove no blocked LeaseSet Create/Edit durable effect, no contaminated retained generation after successful scrub, crash-resumable scrub behavior, and no malformed-SAM secret log echo;
- close as the new safe-partial current-head authority only if no high/medium defect remains.

## Explicit non-goals

This corrective line does not authorize:

- M147/M148 configurable SigType work;
- revival of superseded M149-M151;
- M146 UseOutproxyPlugin implementation;
- legacy AES/LS1 resurrection;
- Yosemite LeaseSet option changes;
- a dedicated LeaseSet secret store;
- changes to M156-M160 ELS2/Red25519/PSK/DH semantics;
- Proposal promotions.

Future full-support work requires a separately accepted architecture/security decision and its own exact-path registration.

## Completion rule

The roadmap closes when M163 removes new and historical durable inert LeaseSet-security state, M164 removes malformed-SAM secret logging, and M165 requalifies the resulting production head with a mechanically correct matrix authority.

If any milestone discovers a new material defect, this roadmap remains active and the finding receives a separately registered corrective successor rather than being waived as terminal.
