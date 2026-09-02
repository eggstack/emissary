# M112 Closure — Client Proxy and Session-Lifecycle Residual Completion

Status: **closed as blocked**

Date: 2026-09-02

Implementation commit: `5b2f3caa6af8767ef393254f20ca010211a8de3a`

Plan: `plans/implementation/i2pcontrol-proposal-170/112-client-proxy-and-session-lifecycle-residual-completion.md`

## Outcome

M112 completed the safe, portable client lifecycle slice in the existing
I2PControl-owned streaming listener. The exact M095 matrix changed from
`288 apply / 94 blocked_primitive / 458 not_applicable` to
`312 apply / 70 blocked_primitive / 458 not_applicable`.

Final M095 matrix SHA-256: `9fea6844e0b7e28959e1169491d100ce2f81124fff790f6c10882b765b41eea9`.

The 24 applied cells are:

- `ConnectDelay` × `client`, `httpclient`, `ircclient`, `socks`, `socksirc`, `connectclient`;
- `Close` × the same six TCP client types;
- `CloseTime` × the same six TCP client types;
- `NewDest` × the same six TCP client types.

The 45 remaining M112-owned blocked cells are:

- `UseOutproxyPlugin`, `SSLProxies`, `JumpList` × `httpclient`, `socks`,
  `socksirc`, `connectclient` (12): no Emissary-owned plugin, TLS-capable
  outbound proxy, or bounded jump/failover owner;
- `Profile` × all seven client families (7): Yosemite 0.7.0 declares no
  portable profile behavior consumed by the accepted session path;
- `DelayOpen` × `streamrclient` (1): no meaningful datagram equivalent;
- `Reduce`, `ReduceCount`, `ReduceTime` × all seven client families (21): the
  accepted Yosemite option declarations have no runtime reduction behavior;
- `Close`, `CloseTime`, `NewDest` × `streamrclient` (3): Streamr remains a
  separate datagram/session owner without the TCP lifecycle events implemented
  here.

No M112 cell was reclassified to `not_applicable`. M111's four `UseSSL` cells
and M113's 21 server presentation/LeaseSet cells remain outside this closure's
applied slice.

## Requirement and evidence matrix

| Work package | Result | Evidence |
|---|---|---|
| WP1 applicability | All 69 starting cells received an exact apply or blocked disposition | `095-full-support-matrix.toml`; `105-residual-option-audit.toml` post-M112 fields |
| WP2 `ConnectDelay` | Applied to six TCP client listener generations; bounded to 0–60,000 ms and cancellable | `runtime/client_listener.rs::ClientStreamConnector::connect`; `connect_delay_is_applied_before_remote_session_use` |
| WP3 `Close*` | `Close`/`CloseTime` applied to six TCP owners; reduction remains blocked | `ConnectionActivity`, `run_idle_closer`, `ClientSessionOwner::close_if_idle`; lifecycle parser tests |
| WP4 Streamr `DelayOpen` | Retained blocked | Matrix note and separate `backends/streamr.rs` ownership boundary |
| WP5 `Profile` | Retained blocked | Yosemite 0.7.0 local `src/options.rs` declaration-only evidence; no portable profile consumer |
| WP6 proxy/plugin/TLS/jump | Retained blocked; existing proxy safety unchanged | `backends/options.rs`; HTTP/SOCKS/IRC proxy-filter tests; no plugin or TLS-MITM path added |
| WP7 ledger/runtime evidence | Completed | `110-completion-ledger.toml`; matrix counts and focused tests |

`NewDest` is accepted only for streaming client types when `Close=true` and
`PersistentClientKey=false`. The first session uses the staged configured
identity; after the owned idle close, the resume options use
`DestinationKind::Transient`. This keeps destination rotation tied to an
actual generation resume and does not rotate identity during staging/manual
start. `Close=true` is rejected for shared sessions because one member cannot
close a session owned by another member.

The implementation is generation-local: state, cancellation, activity guards,
idle closer, and resume options are confined to one listener generation. Session
creation is serialized without holding the bookkeeping lock across network I/O;
idle close waits for active connection tasks, uses bounded monotonic Tokio
timers, and is aborted and awaited on every normal or setup-failure exit.

## Security and containment review

- No proxy/plugin/TLS/jump behavior was enabled, so M093 HTTP/IRC/proxy
  containment remains authoritative.
- Direct `.i2p` destinations still require I2P routing; clearnet destinations
  still require explicit outproxy policy. No direct clearnet DNS fallback or
  request-selected local-target expansion was added.
- No raw destination private key or proxy credential is placed in lifecycle
  diagnostics. `NewDest` uses the existing staged destination store and the
  typed transient resume path.
- No request-controlled plugin/module loading, global timer service, router
  profile registry, core/util change, dependency change, or Yosemite change was
  made.
- Shared session compatibility and Streamr producer isolation remain owned by
  M110/M116 code; M112 rejects the incompatible shared-close combination before
  allocation.

## Verification

The following targeted checks passed during implementation:

- `cargo check -p emissary-cli --no-default-features --features i2pcontrol`;
- lifecycle parser unit test;
- cancellable generation-local `ConnectDelay` test;
- idle close/recreate session test;
- existing client-listener session, cancellation, shared-session, and restart
  tests;
- M095 matrix and M105 audit tests after reconciliation.

`cargo fmt --all -- --check` was attempted. The repository's active stable
rustfmt rejected the repository's existing nightly-only formatting settings and
reported pre-existing formatting differences outside this change; no broad
formatter rewrite was included. `git diff --check` passed.

## Unresolved findings and future handoff

There are no new high- or medium-severity findings. M112 is closed as blocked,
not complete Proposal 170 support. M113 remains proposed/blocked on its exact
server presentation/routing/LeaseSet primitives, and M114 remains blocked on
the zero-residual live/reference final reclosure. M112 did not satisfy or
remove the Yosemite Y003 dependency, so no future plan becomes dependency-ready
from this closure. The registry, README, roadmap, matrix, audit, and ledger
all record this decision and the current `312 / 70 / 458` counts.

## Attestation

This closure is internal to `eggstack/emissary`. External Proposal 170,
reference-router, Yosemite, and documentation sources were read-only evidence.
The user explicitly authorized committing and pushing the resulting internal
changes; no upstream issue, PR, review, release, or maintainer activity was
performed.
