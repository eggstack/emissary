# M097 Closure — Tunnel Common Session and Key Option Completion

Status: blocked

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/097-tunnel-common-session-and-key-option-completion.md`

Source matrix and dependency:

- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`
- M095 closed; its M097-owned cells were reviewed against the current Yosemite/SAM dependency.

Review date: 2026-08-27

## 1. Disposition

M097 is formally closed as blocked. The applicable cells with a demonstrably supported
runtime path are implemented: `TunnelLength`, `TunnelQuantity`, and typed `EncType` are
normalized and passed through one I2PControl-owned session translator into all affected
stream/accepted-server backends. The remaining applicable cells stay blocked with explicit
failure-before-allocation behavior.

This is the stop condition required by M097 §20: current Yosemite 0.7.0's Rust
`SessionOptions` is broader than the actual SAM `SESSION CREATE` serializer, and the
control-plane has no client destination/shared-session authority suitable for the identity
and ownership semantics required here. No core API, dependency fork, or unrestricted path
access was introduced.

## 2. Changed paths

Production changes are confined to the authorized I2PControl boundary:

- `emissary-cli/src/i2pcontrol/domain/tunnel.rs`
- `emissary-cli/src/i2pcontrol/tunnel_manager.rs`
- `emissary-cli/src/i2pcontrol/backends/options.rs`
- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs`
- `emissary-cli/src/i2pcontrol/backends/runtime/client_listener.rs`
- `emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs`
- `emissary-cli/src/i2pcontrol/backends/runtime/mod.rs`
- `emissary-cli/src/i2pcontrol/backends/{client,connect_client,http_bidir,http_client,http_server,irc_client,irc_server,server,socks,socks_irc,streamr}.rs`

Planning and guard changes:

- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`
- `plans/implementation/i2pcontrol-proposal-170/097-tunnel-common-session-and-key-option-completion.md`
- `plans/closure/i2pcontrol-proposal-170/097-closure.md`
- `plans/implementation/i2pcontrol-proposal-170/README.md`
- `plans/registry.md`
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`
- `docs/i2pcontrol/tunnel-manager.md`
- `emissary-cli/tests/m062_dependency_containment.rs`
- `emissary-cli/tests/m095_full_support_matrix.rs`

No `emissary-core/**`, `Cargo.lock`, startup-managed tunnel path, frontend, or dependency
source changed.

## 3. Requirement-to-runtime evidence

| M097 option | Applicable result | Evidence |
|---|---|---|
| `TunnelLength` | apply | typed extraction/range validation; shared translator maps it to inbound and outbound Yosemite lengths; all non-Streamr backends consume the translator |
| `TunnelQuantity` | apply | typed extraction/range validation; shared translator maps it to inbound and outbound Yosemite quantities |
| `EncType` | apply | typed value is validated against current core-supported 3..7 values, max two entries, and the ML-KEM pairing rule; translator sets `lease_set_enc_type` |
| `Shared` | blocked | no bounded compatible-session ownership table and handoff authority exists |
| `UseSSL` | blocked | Yosemite's `SessionOptions.ssl` is not serialized on its SAM `SESSION CREATE` path |
| `TunnelVariance` | blocked | Yosemite exposes the field in Rust but does not serialize it on SAM `SESSION CREATE` |
| `TunnelBackupQuantity` | blocked | Yosemite exposes the field in Rust but does not serialize it on SAM `SESSION CREATE` |
| `SigType` | blocked | Yosemite hardcodes signing type 7 in the destination/session wire path |
| `CustomOptions` | blocked | no bounded allowlisted pass-through to the actual session serializer exists |
| `NewDest` | blocked | no control-plane client destination store and explicit generation-rotation lifecycle exists |
| `PersistentClientKey` | blocked | no restart-safe client identity store and atomic backend handoff exists |
| `PrivKeyFile` | blocked | confined import, key validation, and atomic handoff into a backend-owned key store are not implemented; arbitrary paths remain rejected |

The canonical handler now preserves the common values for edit/get round-tripping, but a
blocked option cannot start a tunnel. `CustomOptions` is bounded to 32 entries and 128-byte
keys/values at ingress. Private-key contents are not copied into canonical output or logs.

## 4. Matrix reconciliation

The M095 guard remains exhaustive at 12 tunnel types and 840 cells. M097's reviewed matrix
totals are:

| Disposition | Cells |
|---|---:|
| `apply` | 115 |
| `planned_apply` | 191 |
| `not_applicable` | 455 |
| `blocked_primitive` | 79 |

The 79 blocked cells are all M097-owned common-option cells or the pre-existing `PrivKeyFile`
cells, with a per-type rationale and named blocking primitive in the authoritative matrix.

## 5. Failure, lifecycle, and security evidence

- Common validation runs before session construction and rejects blocked options without
  allocating a Yosemite session or listener.
- Length and quantity are applied on the next session start/restart; no running session is
  reported as changed by a persisted edit alone.
- The translator uses `DestinationKind` supplied by the existing backend-owned server
  identity path and does not expose private key material.
- Streamr remains on its separate bounded datagram runtime and now validates common fields
  rather than silently accepting them.
- Existing server admission, HTTP filtering, IRC lifetime, and Streamr boundaries remained
  in the feature-gated suite.
- Shared ownership, persistent client restart stability, NewDest rotation, and confined
  `PrivKeyFile` import tests are intentionally not claimed because their required authority
  is the unresolved blocker.

## 6. Verification

Passed:

- `cargo check -p emissary-cli --no-default-features --features i2pcontrol`
- `cargo nextest run -p emissary-cli --no-default-features --features i2pcontrol` — 1711 passed
- M095 matrix and M062 dependency-containment guards — 20 passed
- `git diff --check`

The repository has `m061_containment.rs` and `m062_dependency_containment.rs`; no
`m063_feature_reachability.rs` test target exists in this checkout. The feature-gated suite
includes the existing reachability coverage and passed. The stable formatter reports
pre-existing repository-wide differences caused by nightly-only rustfmt settings
(`wrap_comments`, `imports_granularity`, and related options); this change did not run a
repository-wide rewrite.

## 7. Future-plan disposition

M097 did not unblock M098 or M099: both remain blocked on the unresolved M097 common
session/key authority. M104 remains blocked on M097–M103. M100, M101, M102, and M103 are
independent of these tunnel-option primitives and remain ready. No future plan was silently
activated.

The implementation index, registry, subsystem roadmap, and M095 matrix all record this
transition. A future plan may resume M097 only after it supplies the bounded shared-session
authority, client destination store/rotation lifecycle, confined private-key import, and/or
the missing Yosemite/SAM serializer support named in the matrix.

## 8. Internal-only attestation

The pinned Proposal 170 text and Yosemite 0.7.0 source were read as external evidence only.
No upstream repository, issue, review, merge, adoption request, or contribution channel was
contacted or mutated. All writes remain within the internal Emissary repository.

Disposition: **M097 closed as blocked; supported common session plumbing is applied; M098/M099 remain blocked; M100–M103 remain ready; M104 remains blocked.**
