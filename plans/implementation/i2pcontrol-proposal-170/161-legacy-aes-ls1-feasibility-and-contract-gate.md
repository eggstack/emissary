# M161 — Legacy AES / LeaseSet1 Feasibility and Contract Gate

Status: **closed as complete; outcome B (see `plans/closure/i2pcontrol-proposal-170/161-closure.md`)**

> M160 closed complete (`plans/closure/i2pcontrol-proposal-170/160-closure.md`),
> satisfying this gate's hard dependency. The modern type-5 owner graph
> (no-auth/lookup-secret/PSK/DH) is now frozen with zero Proposal
> promotions.
>
> Registration baseline: `1629b0a5` (M160 implementation/closure head); M161
> registration: `e11b277` (zero-production gate registered). Closure:
> `plans/closure/i2pcontrol-proposal-170/161-closure.md` (outcome B: valid
> legacy-LS1 contract but disproportionate/unsafe under current
> architecture/security policy; `EncryptLeaseSet` remains blocked; zero
> production/dependency changes; M095 remains `336/29/475`).

Class: invariant / compatibility-security feasibility

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Promotion budget: **zero Proposal cells**.
Production budget: **zero production/dependency changes**.

## Objective

Resolve the remaining compatibility question for Proposal `EncryptLeaseSet = "encrypted (aes)"` only after the modern no-auth/lookup/PSK/DH chain is closed, without contaminating modern Encrypted LS2 implementation.

The Proposal-170 PR maps this value uniquely to legacy `i2cp.encryptLeaseSet=true`; it does **not** map legacy AES to `i2cp.leaseSetType=5`. Pinned Java runtime evidence also indicates a material compatibility tension: current `RequestLeaseSetMessageHandler` prefers/requires LS2 when supported, while the legacy session-key encryption operation belongs to the LS1 path. M161 must resolve the actual operational contract instead of assuming that accepting the flag constitutes support.

## Sequencing correction

M161 previously said it hard-depended only on M155 and “may execute after M160.” That was too weak and could allow the legacy gate to race the modern authorization owner freeze.

The corrected dependency is:

```text
M159 PSK primitive -> M160 DH primitive -> M161 legacy AES gate
```

M161 may register only after M160 closes. This guarantees the modern type-5 owner graph is stable before deciding whether legacy LS1 work is coherent or should remain blocked.

## Required evidence

Directly trace and, where practical, exercise:

- Proposal PR `encrypted (aes)` mapping (`i2cp.encryptLeaseSet=true`, not type 5);
- Java I2PTunnel/I2CP LS1-vs-LS2 selection on the pinned snapshot;
- Java `RequestLeaseSetMessageHandler.requiresLS2()` behavior and `_ls2Type` selection;
- legacy `LeaseSet.encrypt(SessionKey)` wire behavior;
- `LeaseSet2`/EncryptedLeaseSet behavior when the legacy flag/key are present;
- current router/client capability negotiation that could still select LS1;
- current Emissary LS1 construction, DatabaseStore, publication, lookup and client-consumption capability, if any;
- whether a pinned reference Java service configured through the Proposal PR with `encrypted (aes)` actually publishes a usable encrypted object against an LS2-capable router;
- whether the Proposal value is operationally coherent in the current reference stack or is a retained configuration surface whose effective data plane is no longer available.

Do not infer support from GUI/property persistence alone.

## Exact path/dependency envelope

M161 is a **zero-production gate**.

Authorized changes are planning/test/evidence only. It may not modify:

- `emissary-core/src/**`;
- `emissary-cli/src/**`;
- Cargo manifests or `Cargo.lock`;
- Yosemite;
- M095/M105 dispositions except through a separately authorized follow-up if authoritative evidence requires a truth correction.

No M061 path amendment or M062 dependency amendment is needed to execute M161 itself beyond advancing the current-registration bookkeeping to a zero-production milestone.

If outcome A requires implementation, M161 must write/register a separate exact-path successor. It must not implement LS1 inside the gate.

## Allowed outcomes

### A — coherent and bounded

Freeze a separate implementation successor for the minimal legacy LS1 publication/runtime required. The successor must identify exact canonical owners, wire fixtures, lookup/client-consumption requirements, key custody, M061/M062 authority and ordinary-LS2 regression boundaries.

### B — valid contract but disproportionate or unsafe under current architecture

Close legacy AES blocked under current architecture/security policy. Modern ELS2 work remains valid, but all five `EncryptLeaseSet` cells remain blocked because the field's complete valid enum domain is not implemented.

### C — reference-incoherent/dead operational value

Record exact contradictory source/runtime evidence. Do not silently reinterpret the value, and do not unilaterally turn it into `not_applicable`; all five `EncryptLeaseSet` cells remain blocked unless the pinned Proposal/reference authority changes or a separately governed applicability re-freeze establishes otherwise.

## Security invariants

- no reinterpretation of legacy AES as modern type-5 ELS2;
- no alias from `i2cp.encryptLeaseSet=true` to modern no-auth/PSK/DH modes;
- no plaintext fallback;
- no broad legacy-protocol resurrection without a dedicated successor;
- no changes to M156-M160 modern primitives;
- no reference test that succeeds only because the flag is ignored counts as support.

## Verification

M161 closure must include:

- source trace with exact pinned commits/paths;
- a minimal reference-runtime exercise if buildable;
- explicit evidence of published DatabaseStore type/object and lookup/decryption behavior, not just controller configuration;
- current Emissary owner inventory;
- `git diff --name-only` proving zero production/dependency changes;
- M061/M062/M095/M105 guards still green;
- unchanged `336/29/475` matrix unless a separate truth-correction plan was required.

## Closure evidence

Record outcome A/B/C, exact source/runtime evidence, any required successor plan, unchanged matrix, zero production diff, and the exact effect on M162's `EncryptLeaseSet` promotion gate.

If outcome A creates a successor, M162 remains deferred until that successor closes. Outcomes B/C allow M162 to proceed with modern field integration, but `EncryptLeaseSet` must remain blocked across all five server families.