# M149 — Encrypted LeaseSet Runtime and EncryptLeaseSet Completion

Status: **superseded / unregistered; do not execute**

This was the original encrypted-LeaseSet draft. It is retained as historical planning context only.

It is superseded by:

- `plans/implementation/i2pcontrol-proposal-170/155-post-m154-leaseset-security-semantic-and-owner-refreeze.md`;
- M156-M162 under `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

## Why superseded

M154 proved the general configurable Destination `SigType` path is blocked, but subsequent reference research established that modern Encrypted LeaseSet2 can use the existing type-7 Ed25519 Destination plus a narrow internally derived type-11 Red25519 blinded key. Therefore M148 completion is not a valid prerequisite for modern encrypted-LS2 infrastructure.

This draft also treated `EncryptLeaseSet` as independently completable before `OptionalLookup` and `LeaseSetClientAuths`. The Proposal's ten-value mode domain directly consumes lookup-secret and PSK/DH authorization primitives, so field-level promotion must be decided only after those lower primitives are complete.

The legacy `encrypted (aes)` value additionally maps to old LS1 `i2cp.encryptLeaseSet` semantics and requires a separate compatibility disposition rather than being conflated with type-5 Encrypted LS2.

## Current authority

M155 is the sole registered continuation. Modern encrypted-LS2 work is decomposed into M156 Red25519/blinding, M157 type-5 publication, M158 lookup-secret/addressing, M159 PSK auth, M160 DH auth, M161 legacy AES feasibility and M162 Proposal integration.

No production path, dependency or matrix promotion is authorized by M149. Historical references to its original intended scope do not override the registry, M061/M062 or the post-M154 corrective roadmap.