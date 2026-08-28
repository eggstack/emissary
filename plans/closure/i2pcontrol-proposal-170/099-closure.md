# M099 Closure — Server Access, Throttle, and LeaseSet Option Completion

Status: **closed internally against the pinned 2026-05-20 Proposal 170 revision; partial**

Closure class: internal-only implementation closure. This record is not an
upstream submission, interoperability certification, or claim of complete
Proposal 170 support.

## Scope and implementation

M099 consumed the closed M098 handoff and reconciled the M095 per-cell matrix.
The implementation remains inside the existing I2PControl server/filter/runtime
owners:

Implementation commit: `b6cc2e0` (`feat(i2pcontrol): complete M099 server
access and throttles`). The closure record and final planning-state updates are
committed separately after this implementation revision.

- `ServerAccessPolicy` canonicalizes full Destinations and base32 hashes once
  during configuration validation. Accepted streams are checked against the
  trusted SAM-derived peer identity before admission and before handler spawn.
- `FilterFilePath` is a bounded newline-delimited peer-access generation. Paths
  must be relative, traversal-free, regular files beneath the server-owned
  administrative root, and within byte/entry limits. A generation is fully
  parsed before use; a failed edit/restart validation cannot replace the prior
  running generation.
- `httpserver`, inbound `httpbidirserver`, `ircserver`, and generic `server`
  now share pre-handler access and bounded admission ownership. HTTP retains
  the existing single request/response sanitizer and now applies `AllowAccept`
  on the server side as well.
- `PerClientPeriod` and `TotalPeriod` bound the peer and aggregate minute
  buckets. `TotalBanTime` expires peer or aggregate denial state locally in the
  current runtime generation. It is not exported to RouterInfo or
  `bannedpeers`.
- LeaseSet `EncryptLeaseSet`, `OptionalLookup`, and `LeaseSetClientAuths` are
  rejected before destination/session allocation. The supported Yosemite/SAM
  path has no exact confidentiality, lookup-policy, or authorization-key
  serializer/handoff, so no silent public downgrade was introduced.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Reconcile every M099 cell before implementation | `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`; `emissary-cli/tests/m095_full_support_matrix.rs` | Pass: independent cells are `apply`; unsafe cells are explicit `blocked_primitive` or `not_applicable`. |
| HTTP presentation/filter options | `backends/filters/http.rs`, `http_server.rs`, `http_bidir.rs`; HTTP backend tests | Pass: existing framing, Host, proxy/privacy, identity, Expect, and response sanitization remain the sole HTTP path. |
| Trusted bounded access control | `backends/runtime/access.rs`; `runtime/accepted_server.rs`; access unit tests | Pass: fixed peer keys, bounded lists, no DNS/network lookup, pre-handler enforcement. |
| Confined filter/access files | `runtime/access.rs`; server/IRC backend validation | Pass: relative-only, traversal/symlink-escape/special-file/size/entry checks and complete-generation parsing. |
| Admission and throttle fields | `runtime/admission.rs`; server/http/bidir/IRC composition; paused-time tests | Pass: finite maxima/periods, monotonic windows, bounded state, RAII leases, generation-local denial expiry. |
| LeaseSet security truthfulness | matrix residual rows; backend rejection paths; M097 closure evidence | Pass: explicit blocked primitive; no downgrade or new core/dependency path. |
| Containment | `emissary-cli/tests/m062_dependency_containment.rs` | Pass: M099 paths and the landed M098 closure are explicitly authorized; no core/util/dependency/frontend/workflow changes. |
| Documentation and operations | `docs/i2pcontrol/{proposal-170-support,proposal-170-conformance,security,tunnel-manager,tunnel-backends}.md`; registry/roadmap/readme | Pass: applied behavior, redaction, residuals, and M104 status are recorded. |

## Matrix and residual disposition

The RouterInfo baseline remains 43 additions: 42 available, 1
protocol-permitted neutral, and 0 unavailable. The M099 server-role cells for
presentation, access, filter, admission, rate, POST, and tunnel-local denial
are `apply` for their matrix-authorized backends. Generic `server` POST fields
are `not_applicable` because its raw stream owner has no HTTP method boundary.

The following six applicable option families remain residual blockers:

1. `AllowInternalSSL`: no supported server-side TLS trust/termination owner;
2. `UniqueLocalAddressPerClient`: no safe per-client local-address allocator
   in the literal-loopback target model;
3. `MultiHoming`: no bounded non-request-selected multihoming/routing owner;
4. `EncryptLeaseSet`: no supported Yosemite/SAM confidentiality serializer;
5. `OptionalLookup`: no supported LeaseSet lookup-policy serializer; and
6. `LeaseSetClientAuths`: no supported authorization key store/session handoff.

They are recorded as `blocked_primitive` with named blocking evidence in the
M095 matrix. They are not converted into fake support by this closure.

## Validation

Commands were run from the repository root:

- `cargo fmt --all` — exit 0; the stable formatter emitted warnings that the
  repository's configured unstable rustfmt options are ignored. The command's
  workspace-wide unrelated formatting output was reverted to preserve the
  containment boundary.
- `cargo check -p emissary-cli --no-default-features --features i2pcontrol` —
  pass.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib` —
  pass, 639 tests.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m062_dependency_containment` —
  pass, 20 tests.
- `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` — pass.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol` —
  pass, 1,756 tests across 25 suites.

`cargo fmt --all -- --check` remains non-green on the pre-existing workspace
format baseline because the stable formatter does not implement the configured
unstable style options. This is tooling/baseline evidence, not an M099 runtime
failure.

## Invariant, security, and recovery review

- Peer accounting uses the trusted fixed 32-byte Destination identity, not
  attacker-controlled headers or long textual keys.
- Access and admission occur before protocol handler work and before local
  target connection. Local targets remain literal loopback addresses.
- Admission leases release global/per-peer occupancy on ordinary return,
  parser failure, panic isolation, cancellation, and shutdown.
- Maps and file generations have explicit limits; denial never evicts active
  or unexpired peer accounting. Time is monotonic and no artificial delay or
  network I/O is introduced.
- Filter parsing and option validation complete before session/destination
  allocation. Secrets and `FilterFilePath` are redacted from ordinary
  canonical responses.
- No router-wide ban source, RouterInfo mutation, core API, dependency change,
  Yosemite patch, or upstream interaction was added.

## Failure, compatibility, and unresolved findings

The first full feature-suite run exposed a stale containment guard that did
not authorize the already-landed M098 closure. The guard was reconciled, and
the complete suite then passed. The stable `cargo fmt --check` mismatch is
retained as a repository tooling limitation described above.

No high or medium M099 regression remains open. The six residual families are
intentional blocked findings, not corrective failures, and remain owned by the
residual option blocker line pending a separately approved bounded primitive.

## Dependent-plan disposition and attestation

M099 is formally closed internally. The active full-support roadmap and
registry now point to the residual option blocker. No future implementation
plan can be unblocked by this work: M104 remains **blocked** until every
applicable residual cell is resolved and live interoperability/security/
containment reclosure is complete. There is no registered executable residual
plan because the missing Yosemite/SAM/key/address-routing owners are not
available inside the accepted containment boundary.

Internal-only attestation: this closure changed only the shared repository
state. No upstream branch, pull request, issue, release, or external service
was modified or submitted.
