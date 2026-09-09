# M151 — LeaseSet ClientAuths Completion

Status: **superseded / unregistered; do not execute**

This original combined client-auth draft is retained as historical planning context only. It is superseded by:

- M155 semantic/owner re-freeze;
- M159 PSK client-authorization primitive;
- M160 DH/X25519 client-authorization primitive;
- M162 Proposal field integration.

## Why superseded

The Proposal/reference modes distinguish PSK and DH authorization with materially different key semantics and runtime cost. Combining both in one implementation milestone would obscure crypto review, bounded-work analysis and interoperability evidence.

M159 now owns PSK authorization; M160 owns X25519/DH authorization. Both are neutral zero-promotion primitives. M162 later decides whether all valid `LeaseSetClientAuths` uses are operational enough to promote the five server-family cells.

No production path, dependency or matrix promotion is authorized by M151. The registry and post-M154 corrective roadmap are current execution authority.