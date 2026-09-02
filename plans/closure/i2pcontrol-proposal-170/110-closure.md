# M110 Closure — Shared Client Session and Destination-Key Ownership

Status: closed

Review date: 2026-09-02

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/110-shared-client-session-and-destination-key-ownership-completion.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Pinned Proposal 170 authority:

- I2P Proposal 170, status `Open`, revision `2026-05-20`.

## 1. Disposition and implementation head

M110 is complete in implementation commit:

- `56c8459ebabd12386a3836d5ccd8e599e5f2936c` — `feat(i2pcontrol): complete shared client identity ownership`

The closure and planning-registry reconciliation are applied after that
implementation commit. The implementation remains confined to the approved
I2PControl composition boundary; no Yosemite, core, util, frontend, startup,
Cargo, or workflow path was changed.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Shared sessions for all seven client families | `backends/runtime/session.rs::SharedClientSessionRegistry`; all seven client backends are composed with it; `client_listener` shared-listener test proves one owner and last-member lifetime | pass |
| Shared compatibility and identity isolation | deterministic key includes all translated `SessionOptions` fields plus a non-reversible private-identity fingerprint; compatibility unit test covers nickname, session-setting, identity, and redaction differences | pass |
| Bounded/concurrent acquisition | per-key creation reservation, `Notify`, 1000-session and 1000-member limits; construction occurs outside the bookkeeping lock | pass |
| Generation-safe teardown | lease-held membership is released by `Drop`; listener/runtime supervisors keep generation-specific tasks and cancellation; a failed new member cannot remove an existing compatible owner | pass |
| NewDest | `ClientDestinationStore::stage` generates a fresh Yosemite destination at the requested generation and commits it only after backend readiness | pass |
| PersistentClientKey | committed client identity is reused across restart fixtures; ephemeral identities are removed on commit and never loaded as persistent state | pass |
| PrivKeyFile | safe relative reference is resolved only below `client-key-imports`, validated before allocation, copied into the owned store, and never used as a mutable runtime path | pass |
| Server-family PrivKeyFile | `ProductionTunnelManagerControl::prepare_server_definition` imports through the existing confined server store for all four applicable server families | pass |
| Streamr separation and bounds | Streamr retains its datagram-specific runtime; shared datagrams use a bounded actor with owned event payloads, never a lock across network I/O; 16 subscribers, 60-second expiry, 1200-byte payload and 4095-byte transport bounds remain explicit | pass |
| Secret and RPC boundaries | `StoredClientDestination` redacts `Debug`/`Display`; private material is held only by the client store/runtime; `raw_config` retains only the safe import reference and boolean options | pass |

## 3. Exact cell-by-cell matrix delta

The authoritative matrix is
`plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`.
Its post-M110 SHA-256 is
`7fa3a6923abab146e060f8ae431e1691905174f1518ac009c94c50528414d94d`.

| Option | Cells moved from `blocked_primitive` to `apply` |
|---|---|
| `Shared` | `client`, `httpclient`, `ircclient`, `socks`, `socksirc`, `connectclient`, `streamrclient` |
| `NewDest` | `client`, `httpclient`, `ircclient`, `socks`, `socksirc`, `connectclient`, `streamrclient` |
| `PersistentClientKey` | `client`, `httpclient`, `ircclient`, `socks`, `socksirc`, `connectclient`, `streamrclient` |
| `PrivKeyFile` | `client`, `httpclient`, `ircclient`, `socks`, `socksirc`, `connectclient`, `server`, `httpserver`, `httpbidirserver`, `ircserver` |

The Streamr server cell remains `not_applicable`, as does the Streamr client
`PrivKeyFile` cell, matching the frozen M105 applicability partition. The
counts changed exactly from `224 / 158 / 458` to `255 / 127 / 458`
(`apply / blocked_primitive / not_applicable`). The machine-readable delta is
also recorded in
`plans/implementation/i2pcontrol-proposal-170/110-completion-ledger.toml`.
The historical M105 record remains a 164-cell audit and now records the
post-M110 reconciliation fields without rewriting its original input evidence.

## 4. Failure, cancellation, restart, and contention review

- Key generation/import is staged before backend allocation and published only
  after the backend reports readiness. Failed starts discard the pending
  identity; failed publication stops the newly started backend.
- Persistent identity publication uses the existing current/backup atomic
  publication primitive, owner-only `0600` files on Unix, bounded payloads,
  regular-file checks, and symlink rejection.
- New destination generation does not reuse a prior identity. Persistent
  identities are reused only when the contract requests persistence.
- Shared stream leases remain in the listener's generation resource until the
  listener exits. Shared datagrams are owned by a single task; bounded command
  and broadcast channels avoid holding locks across Yosemite I/O.
- Per-key creator reservation prevents duplicate equivalent session creation;
  member release removes the registry entry only after the final lease drops.
- Name edits coordinate client identity rename with durable tunnel update and
  compensate the identity rename on update failure. Delete removes the
  control-plane-owned client identity after the tunnel generation stops.
- Import references reject absolute paths, traversal, backslashes, control
  characters, symlink components, special files, oversized files, malformed
  text, and invalid destination material.

## 5. Security, compatibility, and containment review

Changed production paths are limited to:

- `emissary-cli/src/i2pcontrol/backends/{client,connect_client,http_bidir,http_client,http_server,irc_client,irc_server,options,registry,server,socks,socks_irc,streamr}.rs`;
- `emissary-cli/src/i2pcontrol/backends/runtime/{client_listener,mod,session}.rs`;
- `emissary-cli/src/i2pcontrol/{client_secret_store,mod,production,server_secret_store}.rs`.

Tests and evidence changed only in the I2PControl test/planning paths. No
non-I2PControl production path, dependency, router configuration, or protocol
implementation changed. Existing definitions with previously rejected options
remain fail-closed until they are validated and staged through the new owner.
The default feature-disabled path still constructs the historical default
registry; the new owners are composed only by the production I2PControl/SAM
path.

M093 anonymity, loopback/no-SSRF, filtering, server-admission, and secret
boundaries remain intact. No private key is returned in RPC results, runtime
status, errors, `Debug`, or logs. The only persisted request-side path value is
the safe import reference; private destination material is confined to the
owner store and Yosemite runtime handoff.

## 6. Verification outcomes

Passed:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
  664 passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
  2 passed
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

The complete required feature-gated, containment, live-runtime, no-default,
workspace-check, and stable-format verification is run again after this
closure commit and recorded in the final handoff. The repository's known
stable/nightly rustfmt mismatch remains: `cargo fmt --all -- --check` reports
existing unrelated formatting differences and nightly-only option warnings;
formatter churn is not included.

## 7. Future-plan dependency audit

No future plan becomes ready from M110.

- M111 remains `proposed / dependency-blocked` on an accepted public Yosemite
  session-wire path for `UseSSL`, tunnel variance/backups, `SigType`, and
  `CustomOptions`; M110 does not add or imply that primitive.
- M112 remains `proposed / blocked` on its separately named proxy/plugin and
  client lifecycle semantics.
- M113 remains `proposed / blocked` on server presentation/routing and secure
  LeaseSet primitives.
- M114 remains `proposed / blocked` until M111-M113 close as applicable, the
  residual applicable matrix is zero, and final live/reference evidence is
  available.

The registry therefore removes M110 as the active handoff and registers no
successor. Current planning state is partial Proposal 170 support with
`255 / 127 / 458` matrix counts.

## 8. Unresolved findings and final disposition

No M110-scoped implementation finding remains. The pinned Proposal 170
implementation is intentionally not claimed as full support: 127 applicable
cells remain blocked under M111-M113, and M114 remains the only final-reclosure
owner. No safe evidence exists to unblock any of those plans from M110.

M110 is formally **closed** against implementation commit
`56c8459ebabd12386a3836d5ccd8e599e5f2936c`.

## 9. Internal-only attestation

External Proposal 170 material and accepted dependency source were used as
read-only evidence. All repository writes remained within the configured
internal repository. The user explicitly authorized committing and pushing
this implementation and closure to the configured internal `origin`; no
upstream issue, pull request, review, submission, merge, maintainer contact,
release, or contribution preparation was performed.
