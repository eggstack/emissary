# M150 — LeaseSet OptionalLookup Completion

Status: **superseded / unregistered; do not execute**

This original draft is retained as historical planning context only. It is superseded by the post-M154 LeaseSet-security corrective line:

- M155 semantic/owner re-freeze;
- M156 Red25519/blinding;
- M157 modern Encrypted LeaseSet2 publication;
- M158 lookup-secret + blinded-address primitive;
- M162 Proposal field integration.

## Why superseded

The original plan assumed M149 had already completed `EncryptLeaseSet` and then treated `OptionalLookup` as requiring a new generic client-side NetDB lookup/decrypt subsystem.

Reference tracing shows the Proposal server path consumes `OptionalLookup` as the lookup/blinding secret used by encrypted-LS2 publication. A server implementation does not need to invent a second generic NetDB client merely to make that field real; interoperability can be proven with a reference client.

The field must still have real secret-dependent blinding/publication behavior, restart-safe redacted custody, wrong-secret failure and no downgrade. Those requirements now live in M158 and M162.

No production path, dependency or matrix promotion is authorized by M150. The registry and post-M154 corrective roadmap are current execution authority.