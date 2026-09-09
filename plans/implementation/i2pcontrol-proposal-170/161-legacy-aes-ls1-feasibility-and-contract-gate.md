# M161 — Legacy AES / LeaseSet1 Feasibility and Contract Gate

Status: **deferred / unregistered; hard-depends on M155 closure; may execute after M160**

Class: invariant / compatibility-security feasibility

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Promotion budget: **zero Proposal cells**.
Production budget: **zero production/dependency changes**.

## Objective

Resolve the remaining compatibility question for Proposal `EncryptLeaseSet = "encrypted (aes)"` without contaminating modern Encrypted LS2 implementation.

The Proposal PR maps this value to legacy `i2cp.encryptLeaseSet=true` behavior. The pinned Java runtime also forces LS2 when supported, while ordinary `LeaseSet2.encrypt(SessionKey)` is unsupported. M161 must determine the actual operational contract and whether Emissary can support it safely.

## Required evidence

Directly trace and, where practical, exercise:

- Proposal PR `encrypted (aes)` configuration mapping;
- Java I2PTunnel/I2CP LS1-vs-LS2 selection on the pinned snapshot;
- legacy `LeaseSet.encrypt(SessionKey)` wire behavior;
- `LeaseSet2.encrypt(SessionKey)` behavior;
- current router/client capability negotiation that could still select LS1;
- current Emissary absence/presence of LS1 construction, DatabaseStore, publication and lookup support;
- whether a reference Java service configured with this mode actually publishes usable encrypted state against an LS2-capable router.

## Allowed outcomes

### A — coherent and bounded

Freeze a separate implementation successor for the minimal legacy LS1 publication/runtime required. Do not implement it inside M161. The successor must have exact files, wire fixtures, security review and no effect on ordinary LS2.

### B — valid contract but disproportionate/unsafe

Close legacy AES blocked under current architecture/security policy. Modern ELS2 work remains valid, but all five `EncryptLeaseSet` cells remain blocked because the field's full valid value domain is incomplete.

### C — reference-incoherent/dead value

Record exact contradictory runtime evidence. Do not unilaterally reclassify it as N/A; `EncryptLeaseSet` remains blocked unless an authoritative Proposal/reference correction changes the contract.

## Security invariants

- no ad-hoc reinterpretation of legacy AES as type-5 Encrypted LS2;
- no silent aliasing to a modern mode;
- no plaintext fallback;
- no broad legacy protocol resurrection without a dedicated implementation plan;
- no changes to modern M156-M160 primitives.

## Closure evidence

Record outcome A/B/C, exact source/runtime evidence, any required successor plan, unchanged matrix, zero production diff, and explicit M162 effect.