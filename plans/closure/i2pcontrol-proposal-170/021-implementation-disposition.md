# M021 Implementation Disposition — TunnelManager Wire, Atomicity, and Secrets

Status: closed for implementation; M021 closure accepted; M022/M023 ready

Frozen implementation head: `55c4b0f`

Implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/021-tunnelmanager-wire-atomicity-and-secrets.md`

Closure record:

- `plans/closure/i2pcontrol-proposal-170/021-closure.md`

M021 implemented the exact canonical TunnelManager administrative boundary,
one-publication durable mutation semantics, restrictive persistence handling,
legacy secret migration, secret-safe diagnostics and serializers, and explicit
unsupported runtime behavior. The closure record contains the complete
requirement matrix, failure/restart evidence, compatibility/security review,
and verification outcomes.

The only deferred fields are source-dependent runtime destinations and
startup-managed inventory. They remain omitted or neutral rather than
fabricated and are explicitly owned by M023. AddressBook source authority is
owned by M022. No missing tunnel data plane was implemented.

M022 and M023 are now dependency-ready. M024–M027 remain blocked by their
named hard dependencies. Proposal 170 and the subsystem are not finally closed
by this disposition.

All writes remained internal to `eggstack/emissary`; external specification
research was read-only and no upstream interaction was initiated or prepared.
