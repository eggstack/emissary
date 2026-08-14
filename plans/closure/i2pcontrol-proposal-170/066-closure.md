# I2PControl Proposal 170 Milestone M066 — Closure Status

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/066-irc-client-server-tunnel-family.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`

Planning production baseline: `a1296b018ce98d26a019bd5064dff9f4b47e0ad6`.

Implementation commit: this closing commit.

## 1. Executive finding

M066 closes the IRC client/server tunnel family. The implementation is
I2PControl-owned and consumes the M065 local-listener and accepted-stream
seams. It promotes only `ircclient` and `ircserver`; `socksirc` remains an
unsupported backend until M069.

The common IRC filter is bounded, byte-oriented, line-based, and stateful only
per connection. It rewrites identity-bearing client fields, sanitizes
PING/PONG, neutralizes PART/QUIT reasons, permits ordinary IRC and CTCP ACTION,
and drops unsupported CTCP including all requested DCC modes. The server role
has an independent bounded registration gate and does not connect the local
IRCd until NICK and a peer-derived USER have been accepted.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Common parser is bounded and byte-oriented | `backends/filters/irc.rs`; `MAX_IRC_LINE = 2048`, CRLF normalization, tags/prefixes, control/NUL rejection | pass |
| Client USER hostname leakage is prevented | common filter rewrites the third USER field to `0`; table test and duplex relay test | pass |
| PING/PONG address leakage is prevented | per-connection expected-token state rewrites both directions to `emissary-ping`; isolated-state test | pass |
| PART/QUIT identifying text is neutralized | fixed `leaving` reason in both client commands; table tests | pass |
| Ordinary PRIVMSG/NOTICE works | explicit command policy and ordinary-message fixture | pass |
| CTCP ACTION works | exact CTCP classification fixture | pass |
| Unsupported CTCP/DCC fails closed | VERSION/TIME and DCC CHAT/SEND/RESUME/ACCEPT fixtures drop without logging or auxiliary setup | pass |
| Malformed framing fails closed | overlong, NUL, malformed CTCP, and unknown-command fixtures | pass |
| IRCv3 tags/CAP/SASL compatibility | tags/prefix parser fixture and CAP/AUTHENTICATE command policy | pass |
| `ircclient` uses M065 local listener/stream seam | `irc_client.rs` composes `run_client_listener`, `ClientStreamConnector`, and `relay_client_stream` | pass |
| `ircserver` uses accepted streams and trusted identity | `irc_server.rs` composes `run_accepted_server`; handler reads `AcceptedServerConnection.peer` | pass |
| Server registration occurs before local target connect | bounded `read_registration` completes before `TcpStream::connect`; duplex registration tests | pass |
| Registration bounds are explicit | 5-second line timeout, 15-second total timeout, 1,024-byte line cap, 12-line cap | pass |
| Cross-protocol probes are rejected | HTTP method/HTTP2 and BitTorrent-like first-line signatures are rejected before target connect | pass |
| USER identity uses trusted peer material | server hostname comes from `TrustedPeerIdentity::destination()` through deterministic B32 conversion; spoofed host is ignored | pass |
| Local server target is confined | default loopback; only `127.0.0.1` and `localhost` accepted | pass |
| Relevant options are applied or rejected before allocation | `IRC_CLIENT_OPTIONS` and `IRC_SERVER_OPTIONS`; automation, access/auth, I2CP, custom, WEBIRC/cloak, and DCC options reject | pass |
| Secrets and private IRC text are not logged | no raw-line diagnostics; password test asserts rejection output omits the secret | pass |
| Per-connection state is bounded and isolated | one `IrcFilter` behind one connection-local mutex; no nickname/global state | pass |
| Lifecycle is generation-safe and cancellable | per-name supervisors mirror M065 reserve/set-task/complete/stop generation rules; lifecycle backend tests | pass |
| Persistence/restart server identity is retained | production composition prepares `ircserver` with the existing fixed server secret store and persists public destination | pass |
| Registry promotion is limited | production registry maps `ircclient` and `ircserver`; `socksirc` and other families remain unsupported | pass |
| Source and feature containment remain intact | all new production logic is under `emissary-cli/src/i2pcontrol/**`; no core or manifest changes | pass |

## 3. Filter behavior matrix

| Traffic | Policy |
|---|---|
| PASS/NICK/USER/CAP/AUTHENTICATE and reviewed IRC commands | pass, with USER hostname rewrite |
| PRIVMSG/NOTICE ordinary text | pass |
| CTCP ACTION | pass |
| CTCP VERSION/TIME/unknown and malformed CTCP | drop |
| DCC CHAT/SEND/RESUME/ACCEPT | drop; no auxiliary tunnel |
| Client PING and server PONG | rewrite to connection-local fixed token |
| Server PING and client PONG | rewrite to connection-local fixed token |
| Client PART/QUIT reason | rewrite to fixed neutral reason |
| Unknown client command | drop |
| Malformed, overlong, NUL/control-bearing line | close/drop according to parser policy |

No filter diagnostic includes raw IRC lines, passwords, tokens, or private chat
text.

## 4. Server registration evidence

The registration gate uses `REGISTRATION_LINE_TIMEOUT = 5s`,
`REGISTRATION_TIMEOUT = 15s`, `MAX_REGISTRATION_LINE = 1024`, and
`MAX_REGISTRATION_LINES = 12`. It requires NICK and USER, allows only the
reviewed registration commands, rejects the first-line HTTP/HTTP2 and
BitTorrent-like signatures, and forwards rewritten registration only after all
checks pass. The USER hostname is calculated from the accepted stream's
SAM-derived public destination, not from the remote IRC field. Target host and
port are configuration-only and loopback-confined.

## 5. Option-capability disposition

| Option family | `ircclient` | `ircserver` |
|---|---|---|
| Listen interface/port | `ReachableBy` IP plus required `Port` | `Port` is the local IRCd target port; no listener interface |
| I2P target destination/port | required `TargetDestination`; optional `TargetPort`, direct Yosemite destination only | not applicable |
| Local target host/port | not applicable | loopback `TargetHost`/`Host` default plus `TargetPort` or `Port` |
| Access/auth fields | rejected; no auth owner in M066 | rejected; accepted-stream access policy is not silently implied |
| IRC server/port/nick/password/channels | rejected as unimplemented automation | rejected as unimplemented automation |
| I2CP/session fields | rejected; M065 IRC config has no I2CP mapping | rejected; M065 accepted-server config has no I2CP mapping |
| Custom/WEBIRC/cloak options | rejected | rejected |
| DCC-related options | rejected; DCC payload is also filtered | rejected |

Password values are never included in errors or diagnostics.

## 6. Lifecycle and failure disposition

- validation runs before listener, session, destination lookup, or local target allocation;
- one connection's filter, connect, parse, or relay failure closes only that connection;
- sibling connection tasks remain owned by the bounded M065 task group;
- stop signals the exact named generation and drains its current-generation handler tasks;
- duplicate starts are rejected and stale completion cannot update a restarted generation;
- server local-target failure closes the accepted stream without disclosing target details over I2P;
- no startup-managed IRC resource was added or adopted.

## 7. Changed paths

Production:

- `emissary-cli/src/i2pcontrol/backends/filters/mod.rs`
- `emissary-cli/src/i2pcontrol/backends/filters/irc.rs`
- `emissary-cli/src/i2pcontrol/backends/irc_client.rs`
- `emissary-cli/src/i2pcontrol/backends/irc_server.rs`
- `emissary-cli/src/i2pcontrol/backends/options.rs`
- `emissary-cli/src/i2pcontrol/backends/registry.rs`
- `emissary-cli/src/i2pcontrol/backends/mod.rs`
- `emissary-cli/src/i2pcontrol/production.rs`

Tests, support documentation, planning, and closure evidence:

- `emissary-cli/tests/m062_dependency_containment.rs`
- `docs/i2pcontrol/proposal-170-support.md`
- `docs/i2pcontrol/tunnel-manager.md`
- `plans/002-long-term-roadmap.md`
- `plans/registry.md`
- `plans/implementation/i2pcontrol-proposal-170/066-irc-client-server-tunnel-family.md`
- `plans/implementation/i2pcontrol-proposal-170/069-socks-and-socks-irc-tunnels.md`
- `plans/implementation/i2pcontrol-proposal-170/README.md`
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`
- `plans/closure/i2pcontrol-proposal-170/066-closure.md`

No `emissary-core/**`, Cargo manifest, lockfile, startup proxy, CI, fuzz,
coverage, release, or upstream/third-party path changed.

## 8. Verification executed

```text
cargo fmt --all
cargo test -p emissary-cli --no-default-features --features i2pcontrol backends::filters::irc
cargo test -p emissary-cli --no-default-features --features i2pcontrol backends::irc
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core --no-default-features
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --lib -- -D warnings
git diff --check
```

Focused IRC/filter, backend, clippy, and feature-enabled check results passed
from the final working tree. `git diff --check` also passed. The stable
toolchain's `cargo fmt --all -- --check` reports formatting differences in
pre-existing files because this repository's rustfmt configuration uses
nightly-only options; running the formatter would rewrite unrelated paths, so
those paths were preserved. The two historical containment tests also report
their pinned-baseline drift beginning with pre-existing paths such as
`emissary-cli/src/address_book.rs`, outside the M066 change set. Finally,
`cargo check -p emissary-core --no-default-features` retains the inherited
feature-disabled `RwLock` import failure in unrelated core paths. M066 adds no
core, manifest, or unrelated production paths, and none of these qualifications
identify an M066-specific violation.

## 9. Successor disposition

M067, M068, M069, and M071 are now dependency-ready. Per the repository rule
that only one next handoff is registered as ready, M067 is marked `ready` in
the implementation README, subsystem roadmap, and `plans/registry.md`.
M068, M069, and M071 are marked blocked only because they are not the next
registered handoff. M070 remains blocked on M067 and M068; M072 remains blocked
on M066-M071. M051's independent RouterInfo blocker is unchanged.

Internal-only attestation: all repository writes are scoped to
`eggstack/emissary`. No upstream or third-party issue, review, merge,
submission, contribution package, or maintainer contact was prepared.

Unresolved M066 findings: none. The verification qualifications above are
inherited repository-baseline/toolchain issues, not M066 findings. DCC
auxiliary tunnels, WEBIRC secrets, configurable cloaks, IRC automation,
`socksirc`, and public-network IRC interoperability remain explicitly deferred
to their stated future scopes.

Final disposition: **closed**.
