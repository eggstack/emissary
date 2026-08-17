# M077 Closure — IRC Server Lifetime and Exhaustion Hardening

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/077-irc-server-lifetime-and-exhaustion-hardening.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`

Implementation commit:

- `0660ca6` — feat(i2pcontrol): harden IRC server lifetime

## 1. Disposition

M077 is closed. The accepted IRC server path now bounds local IRCd connection
establishment to five seconds and replaces the unbounded post-registration
`io::copy` pair with an I2PControl-local bidirectional relay whose deterministic
ten-minute inactivity deadline resets after successful traffic in either
direction. The relay remains byte-transparent after registration, owns no
additional admission policy, and returns through the existing accepted-server
task boundary so the M074 admission lease is released on completion, error,
panic isolation, cancellation, or abort.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Ten-minute post-registration inactivity ceiling | `POST_REGISTRATION_INACTIVITY`; `relay_with_inactivity`; paused-time expiry test | pass |
| Activity resets the deadline | Watch-based activity notification after each successful read/write; remote-to-local and local-to-remote paused-time tests | pass |
| No fixed total connection lifetime | Three traffic exchanges over more than 20 minutes of paused time keep the relay alive | pass |
| Raw post-registration semantics remain intact | Relay copies bounded byte buffers without IRC parsing; duplex tests verify exact bytes in both directions | pass |
| Registration filter remains before local connect | Existing bounded registration parser/rewrite tests and handler ordering: parse, connect, write sanitized registration, then relay | pass |
| Trusted peer hostname rewrite is preserved | Existing `registration_rewrites_trusted_peer_and_rejects_http` test and unchanged `rewrite_server_user` path | pass |
| Malformed/wrong-protocol registration remains fail-closed | Existing HTTP probe and registration-bound tests; no target connect is reachable before parser success | pass |
| Local target connection is bounded to five seconds | `TARGET_CONNECT_TIMEOUT`, generic paused pending-future test for `bounded_connect` | pass |
| Local target errors are sanitized and not remotely exposed | Connect failures are converted to a static error and the handler closes quietly; test asserts no host, port, or OS detail | pass |
| One backend failure does not fail the server runtime | Handler result remains isolated inside the existing accepted-server task; runtime supervisor continues and existing lifecycle tests pass | pass |
| M074 shared admission is reused | `IrcServerConfig.admission` continues through `run_accepted_server`; no IRC limiter or second admission component added | pass |
| Admission releases on idle expiry | Paused-time test holds an admission lease around the relay, advances ten minutes, then reacquires the single permit | pass |
| Admission releases on EOF/error | Relay returns on remote and local EOF; the accepted-server task owns the existing RAII lease, and M074 admission release tests pass | pass |
| Stop/restart cancellation remains bounded and generation-safe | Existing `stop_generation`/`BoundedTaskGroup::drain` path, lifecycle restart test, and unchanged ten-second supervisor stop bound | pass |
| No lock crosses connect/relay | Admission and supervisor locks are released before handler I/O; relay has no shared mutex | pass |
| No new core API, dependency, or public protocol field | Production diff is limited to `emissary-cli/src/i2pcontrol/backends/irc_server.rs` | pass |
| Runtime documentation is current | Proposal 170 support and TunnelManager docs describe five-second connect and ten-minute activity-resetting expiry | pass |

## 3. Production implementation evidence

`handle_accepted_connection` still validates the accepted Yosemite peer,
performs the existing bounded registration read and trusted `USER` rewrite,
connects only to the configured loopback IRCd, writes only sanitized
registration lines, and then enters the raw relay. `bounded_connect` maps both
refusal/error and timeout to one static local-target error. The relay uses two
small read/write loops and a coalescing Tokio watch notification, so activity
tracking adds bounded state and no per-message IRC parser or unbounded event
queue.

The existing accepted-server runtime remains the lifecycle owner. Its task
group drains handlers during stop and aborts them after the established bounded
wait; each handler task retains its M074 `AdmissionLease` until the handler
returns or is cancelled.

## 4. Verification executed

### Commands run

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol irc_server
cargo test -p emissary-cli --no-default-features --features i2pcontrol runtime::admission
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
rustfmt +nightly --check --edition 2021 --config-path rustfmt.toml emissary-cli/src/i2pcontrol/backends/irc_server.rs
git diff --check
```

### Results

All final commands passed. The feature-enabled package suite passed 1,574
tests across 24 suites. The focused IRC run passed 22 tests; the admission run
passed 12 tests; the M061 and M062 containment suites passed 7 and 19 tests.
Both CLI checks, the core check, strict Clippy, scoped nightly rustfmt, and
the final whitespace check passed.

The repository-wide stable `cargo fmt --all` configuration remains affected by
the inherited nightly-only rustfmt options and rewrites unrelated files; those
formatter-only changes were discarded. The touched Rust file passes the
repository's scoped nightly rustfmt check.

## 5. Invariant review

- The M066 registration parser, line/time bounds, wrong-protocol rejection,
  CAP/PASS/AUTHENTICATE/PING/PONG compatibility, and trusted hostname rewrite
  are preserved.
- The inactivity timer is a resettable resource deadline, never a total
  connection timeout. Deterministic ten-minute expiry is used; no jitter or
  artificial delay was added.
- Relay bytes are forwarded without post-registration IRC parsing or framing
  changes. EOF and inactivity drop both relay directions by dropping their
  owned halves.
- Admission remains M074-owned and is released through the existing lease
  lifetime. No lock is held across network I/O or task joining.
- Production changes remain under the I2PControl boundary, with no core/router
  path, dependency, public field, WEBIRC, or DCC addition.

## 6. Failure and recovery review

Malformed or incomplete registration returns before local allocation. A target
refusal or timeout closes only that accepted handler and does not return an
accepted-server runtime error. Remote EOF, local EOF, relay I/O failure,
inactivity expiry, panic isolation, and handler cancellation all terminate the
handler task; the existing RAII lease and bounded task-group shutdown release
capacity. Supervisor generations remain exact across stop and restart.

## 7. Migration and compatibility review

No persistent schema, Proposal 170 wire contract, tunnel option, or public API
changed. Existing registered IRC sessions remain active while traffic
continues in either direction. The only new behavior is expiry of a registered
session that has been idle for ten minutes, matching the reference scale.

## 8. Security review

The trusted I2P destination remains the sole source of the presented IRC
hostname. Local target selection remains fixed loopback configuration, and
target connection failures expose no host, port, OS, filesystem, or backend
detail. M074 peer-aware admission remains the only server admission mechanism;
the new idle bound limits distributed/Sybil idle occupancy without adding a
second IRC-specific quota.

No high- or medium-severity IRC exhaustion or anonymity finding remains in the
M077 scope.

## 9. Documentation and operations

Updated:

- `docs/i2pcontrol/proposal-170-support.md`
- `docs/i2pcontrol/tunnel-manager.md`
- M077 implementation status
- the tunnel-security roadmap and active registry
- M078 successor status to `ready`

The implementation and closure commits are internal repository changes only.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| — | None in M077 scope | — | — |

## 11. Roadmap disposition

Milestone closed and the next dependency may proceed. M078 is now the single
registered `ready` handoff. M079 remains blocked until M078 closes; unrelated
M051 source/truthfulness work remains independently blocked.

## 12. Registry updates

Updated `plans/registry.md`,
`plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`,
`plans/implementation/i2pcontrol-proposal-170/README.md`, and the M077/M078
implementation statuses. M077 is recorded closed, M078 is recorded ready,
and M079 remains blocked on M078.

## 13. Internal-only external interaction attestation

Java I2P and pinned Proposal 170 reference material were used only as
read-only behavioral evidence. No upstream repository, maintainer channel,
issue, pull request, review, merge, adoption, or submission was mutated or
requested. The push is limited to the internal Emissary repository remote.
