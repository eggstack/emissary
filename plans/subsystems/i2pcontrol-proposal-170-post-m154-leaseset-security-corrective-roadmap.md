# I2PControl Proposal 170 Post-M154 LeaseSet-Security Corrective Roadmap

Status: **active / partial; M155 is the only registered successor**

This roadmap supersedes the LeaseSet-security execution ordering in the older residual and post-M146 roadmaps while preserving M146/M147/M154 closures as historical evidence.

Current baseline:

- `master` at roadmap creation: `5a9754cdd2226ed23336feb61e6fa77118d2939e`;
- current whole-surface qualification authority: M153;
- matrix: `336 apply / 29 blocked_primitive / 475 not_applicable`;
- remaining blockers: 10 `SigType`, 5 `EncryptLeaseSet`, 5 `OptionalLookup`, 5 `LeaseSetClientAuths`, 4 `UseOutproxyPlugin`;
- M146 closed blocked with four outproxy-provider cells;
- M147/M148 configurable Destination `SigType` path closed blocked by M154 disposition C.

## 1. Corrective finding

M154 correctly blocked a general configurable destination-signature-suite primitive, but that does not imply modern Encrypted LeaseSet2 is blocked behind the same primitive.

The modern encrypted-LS2 path may start from the existing type-7 Ed25519 Destination and derive a type-11 Red25519 blinded signing key. This is a narrower cryptographic primitive than user-selectable/persistent Destination `SigType` and must be planned separately.

The old M149 -> M150 -> M151 ordering also over-coupled the Proposal fields. `EncryptLeaseSet` is a ten-value mode selector whose lookup-password and PSK/DH modes consume `OptionalLookup` and `LeaseSetClientAuths`. The modern primitives therefore have to be established before final field-level promotion decisions.

The legacy `encrypted (aes)` value is a separate compatibility question because it maps to old `i2cp.encryptLeaseSet` behavior rather than type-5 Encrypted LS2. It must not silently block development of modern encrypted LS2, but it may continue to block the five `EncryptLeaseSet` cells if its valid contract cannot be implemented safely.

## 2. Containment rules

- M061/M062/M093 remain binding.
- Proposal policy and secret/config mapping stay in `emissary-cli/src/i2pcontrol/**` where possible.
- Red25519, blinding, encrypted-LS2 serialization/publication, lookup-secret derivation and auth crypto must be neutral lower-layer primitives with Proposal-free APIs.
- Every production milestone requires exact-file authorization at registration; no broad crypto/netdb/i2np/destination prefix authorization.
- No downgrade from requested encrypted/authenticated publication to ordinary plaintext LeaseSet2.
- Secret, PSK, DH and blinding material is never response-facing or logged.
- M147 remains blocked; no DSA/P256/P384/P521 generation is smuggled into this line.
- M146 remains blocked; no clearnet egress/provider work is mixed into this line.

## 3. Dependency graph

```text
M153 current-head requalification                   [CLOSED]
  |
  v
M154 SigType domain/security refreeze               [CLOSED; M147 PATH BLOCKED]
  |
  v
M155 LeaseSet semantic/owner refreeze               [REGISTERED; ZERO PRODUCTION]
  |
  v
M156 narrow Red25519/blinding primitive             [DEFERRED; ZERO PROMOTION]
  |
  v
M157 modern Encrypted LeaseSet2 publication         [DEFERRED; ZERO PROMOTION]
  |
  v
M158 lookup-secret + blinded-address primitive      [DEFERRED]
  |
  v
M159 PSK client-authorization primitive             [DEFERRED; ZERO PROMOTION]
  |
  v
M160 DH client-authorization primitive              [DEFERRED; ZERO PROMOTION]
  |
  +--> M161 legacy AES/LS1 feasibility              [DEFERRED; ZERO PRODUCTION]
  |
  v
M162 Proposal LeaseSet-field integration            [DEFERRED; CONDITIONAL PROMOTIONS]
  |
  v
M152 final residual requalification                 [DEFERRED; ZERO PROMOTION]
```

Only the next dependency-ready milestone may be registered.

## 4. Milestone intents

### M155 — semantics and owner re-freeze

Freeze the ten-value Proposal mapping, legacy AES disposition, type-7 -> type-11 blinding requirement, field coupling, exact owners and dependency policy. Zero production and zero matrix change.

### M156 — narrow Red25519/blinding

Implement only the standard Ed25519-to-Red25519 blinding/signing operations needed by encrypted LS2. This is not a generic signature-suite framework and does not make type 11 a persistent Destination type. Zero Proposal promotions.

### M157 — modern encrypted-LS2 publication

Implement type-5 Encrypted LeaseSet2 construction, nested encryption, blinded storage key, DatabaseStore type, publication/storage verification and UTC-day rollover using real current leases. No Proposal field promotion yet.

### M158 — lookup secret and blinded address

Implement optional secret contribution to blinding plus standard extended encrypted-service `.b32.i2p` encoding/decoding needed for usable/interoperable lookup. Whether the five `OptionalLookup` cells promote here or at M162 is decided by M155's field-completeness rule; default is defer promotion to M162.

### M159 — PSK client authorization

Implement bounded PSK authorization entries and encrypted-LS2 layer-1 client-cookie construction with known-answer/reference interoperability. Zero Proposal promotions.

### M160 — DH client authorization

Implement bounded X25519 DH authorization with per-client work limits and reference interoperability. Zero Proposal promotions.

### M161 — legacy AES/LS1 feasibility

Resolve whether the Proposal's `encrypted (aes)` value has a coherent current reference runtime and, if so, freeze the smallest legacy LS1 publication implementation plan. M161 itself has zero production. If a real LS1 implementation is required, it creates a separate implementation successor rather than hiding it inside M162.

### M162 — Proposal integration

Integrate the real primitives with I2PControl server families and promote only fields whose entire valid contract is operational.

Expected maximum promotions if modern modes, OptionalLookup and auth are fully implemented but legacy AES remains unavailable:

- `OptionalLookup`: up to 5;
- `LeaseSetClientAuths`: up to 5;
- `EncryptLeaseSet`: 0 unless all ten valid values are operational or an authoritative contract correction removes the missing value.

That yields a possible safe-partial state of `346/19/475` before final requalification. If legacy AES is also implemented, `EncryptLeaseSet` may add 5 more, reaching `351/14/475`.

These counts are planning ceilings, not claims.

### M152 — final whole-surface requalification

Recompute all 840 cells and establish final safe-partial/full status. M152 may never promote cells by itself.

## 5. Old-plan disposition

M149, M150 and M151 remain useful historical drafts but are **superseded for execution** by M155-M162 because their dependency ordering assumes M148 completion and treats the three LeaseSet fields too independently.

M152 remains the final requalification milestone, but its entry gate is re-based to M162 closure (plus any M161-created legacy-AES implementation successor, if required).

M147/M148 remain blocked and are not reopened by this roadmap.

## 6. Completion conditions

This corrective line is complete when:

- M155-M162 have closed according to their gates;
- modern encrypted-LS2 publication/auth/lookup primitives are either implemented or truthfully blocked with exact evidence;
- legacy AES has an explicit supported/blocked disposition rather than being conflated with LS2;
- M152 requalifies the actual current head;
- M095/M105/registry/roadmaps agree exactly;
- any remaining blockers are explicit terminal policy/architecture blockers, not missing planning.