# M090 Closure — Server Loopback and IRC Half-Close Corrective

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/090-server-loopback-and-irc-half-close-corrective.md`

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Planning baseline: `f0f3fc2204318c2fac69817d347df2702c51287b`.

Implementation head: `172a4e86d0d183c028244b02e91440ac36525c0c`.

Review date: 2026-08-26.

## 1. Disposition

M090 is complete. The HTTP server family and IRC server now normalize their
accepted local-target compatibility spellings to literal loopback `IpAddr`
values before connection. IRC now preserves useful TCP half-close/drain
behavior while retaining its ten-minute progress-based inactivity bound.

The correction stayed within the approved `i2pcontrol` boundary and changed no
Proposal 170 wire/API surface, dependency, core/router behavior, startup
ownership, frontend state, Streamr behavior, or `httpbidirserver` identity
semantics.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| HTTP accepted targets are literal addresses before connect | `HttpServerConfig.target_address` is `IpAddr`; `handle_http_stream` connects with `SocketAddr::new`; normalization tests cover `localhost`, `127.0.0.1`, and `::1` | pass |
| Inbound HTTP-bidirectional server follows the shared typed seam | `HttpBidirConfig.target_address` uses `normalize_loopback_target` and passes the `IpAddr` to `make_accepted_handler`; existing composition tests remain green | pass |
| IRC accepted targets are literal addresses before connect | `IrcServerConfig.target_address` is `IpAddr`; `connect_local_target` connects with `SocketAddr::new`; `localhost` normalization test passes | pass |
| No resolver-dependent `localhost` reaches corrected connects | HTTP, HTTP-bidir, and IRC handlers receive only typed `IpAddr` values; no corrected path calls `TcpStream::connect` with a hostname | pass |
| Non-loopback targets fail closed before runtime/destination lookup | HTTP and IRC configuration tests reject `10.0.0.1`; validation remains in `config_without_destination` before persistent destination lookup | pass |
| Existing compatibility surface is preserved | `localhost` and existing IPv4 loopback remain accepted; HTTP/HTTP-bidir retain existing `::1` support, while IRC retains its prior IPv4-only accepted surface | pass |
| IRC remote-to-local EOF permits local-to-remote drain | `remote_eof_allows_local_to_remote_drain` sends a response after remote half-close and verifies delivery | pass |
| IRC local-to-remote EOF permits remote-to-local drain | `local_eof_allows_remote_to_local_drain` sends a request after local half-close and verifies delivery | pass |
| IRC inactivity remains progress-based and ten minutes | Existing `registered_idle_peer_expires_and_releases_admission`, `activity_resets_idle_deadline_without_fixed_lifetime`, `traffic_in_either_direction_resets_idle_deadline`, and `inactivity_closes_both_relay_directions` pass unchanged in semantics | pass |
| Admission ownership is released after completion/expiry | `half_close_completion_releases_admission_lease` reacquires the bounded peer slot after relay completion; idle expiry test covers the timeout path | pass |
| Identity, HTTP behavior, and secret ownership remain unchanged | Only target representation, the IRC relay state machine, and focused tests changed; accepted-server ordering, HTTP filters/POST limiter, destination store, and diagnostics were not modified | pass |
| Scope and API containment hold | Implementation commit changes only `http_server.rs`, `http_bidir.rs`, and `irc_server.rs`; M062 exact-path guard passes | pass |

## 3. Implementation evidence

The shared helper accepts only the pre-existing spellings needed by each
backend. `localhost` maps directly to IPv4 loopback, and HTTP-family `::1`
maps directly to IPv6 loopback. Invalid values are rejected before the backend
loads or allocates a persistent server Destination. Runtime connects receive a
`SocketAddr`, so resolver/NSS/DNS behavior cannot influence local-target
selection.

IRC relay directions are task-local futures with independent active flags. A
direction that reaches EOF shuts down its opposite writer and is then disabled;
the other direction continues until it completes, errors, or reaches the
shared inactivity deadline. Only successful `read` plus `write_all` progress
advances the activity sequence and resets the deadline. No absolute connection
age limit was introduced.

## 4. Failure, cancellation, recovery, and contention review

- Invalid target options fail before destination/session/runtime allocation.
- Literal loopback connect failures remain bounded by the existing five-second
  timeout and return the existing sanitized failure behavior.
- EOF, relay I/O errors, inactivity expiry, handler cancellation, and stop
  continue through the existing accepted-server task ownership; the admission
  lease is held by the handler and drops on completion.
- Half-close completion is explicitly tested for lease release. Timeout-based
  lease release remains covered by the existing paused-time test.
- The relay owns no shared mutex and does not hold one across socket I/O,
  shutdown, timers, or task joins. No detached tasks were added.
- Generation-local supervisor state, trusted peer identity ordering, bounded
  task groups, and restart/stop behavior are unchanged.
- No new diagnostics or error paths include peer or private Destination text.

## 5. Compatibility, migration, and security review

There is no schema, persistence, dependency, feature, wire/API, or migration
change. Existing options retain their spelling and fail-closed policy. The
local target is now stronger than hostname validation because it is represented
as an address before any accepted stream can connect. HTTP parsing, response
filtering, POST accounting, IRC registration, raw post-registration bytes,
Streamr, and bidirectional identity/session behavior are unchanged.

The lower-layer/pre-accept admission limitation remains unresolved and is not
claimed as fixed by M090. M091 remains the bounded owner of that future issue.

## 6. Changed paths and containment

Implementation commit:

- `emissary-cli/src/i2pcontrol/backends/http_server.rs`;
- `emissary-cli/src/i2pcontrol/backends/http_bidir.rs`;
- `emissary-cli/src/i2pcontrol/backends/irc_server.rs`.

Planning and closure bookkeeping:

- `plans/implementation/i2pcontrol-proposal-170/090-server-loopback-and-irc-half-close-corrective.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/registry.md`;
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`;
- `plans/closure/i2pcontrol-proposal-170/090-closure.md`.

The M062 exact planning allowlist already explicitly contained the M090/M091
plan and closure paths; no production glob or dependency ownership was
broadened. No path outside `emissary-cli/src/i2pcontrol/**` changed in the
implementation commit.

## 7. Verification

| Command | Outcome |
|---|---|
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol http_server::tests` | pass; 26 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol irc_server::tests` | pass; 28 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | pass; 1696 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment` | pass; 19 tests |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass; no issues |
| `git diff --check` | pass |
| `git diff --name-only f0f3fc2204318c2fac69817d347df2702c51287b` / `git diff --stat f0f3fc2204318c2fac69817d347df2702c51287b` | reviewed; implementation paths are the three listed above; planning baseline also contains the pre-existing M090/M091 planning records |
| `cargo fmt --all -- --check` | not green under the repository's current stable formatting setup; it reports pre-existing repository-wide formatting drift caused by nightly-only rustfmt settings. The formatter-only changes were not retained. |

The same formatting incompatibility was confirmed with the installed nightly
toolchains; it is unrelated to M090 semantics and does not alter the clean
`git diff --check` result.

## 8. Future-plan disposition

M091 remains **blocked**, not ready. Its hard sequencing dependency on M090 is
now satisfied, but its substantive blocker remains: the current Yosemite/SAM
boundary exposes no supported streaming-concurrency configuration transport,
and Emissary's declared lower-layer fields are not consumed by the streaming
manager. A dependency, Yosemite fork/vendor/git change, manifest/lockfile
change, or `emissary-core/**` expansion still requires explicit authorization
before M091 can move to ready.

No other future tunnel-security plan was unblocked. The independent current-
head security reclosure remains unregistered until M091 is closed. M051 and
the accepted RouterInfo 37/1/5 and unrelated AddressBook/base-I2PControl
dispositions remain unchanged.

## 9. Internal-only attestation

All implementation, testing, closure, and planning writes were confined to the
internal `eggstack/emissary` repository. No upstream issue, pull request,
review, submission, merge request, maintainer contact, contribution artifact,
or external repository write was opened, drafted, requested, or pushed.
External specifications and reference sources remain read-only evidence.
