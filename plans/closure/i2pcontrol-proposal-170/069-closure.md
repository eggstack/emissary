# I2PControl Proposal 170 Milestone M069 — Closure Status

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/069-socks-and-socks-irc-tunnels.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`

Planning production baseline: `a1296b018ce98d26a019bd5064dff9f4b47e0ad6`.

Implementation commit:

- `3a9a4a8eb913fa60081404790923b216b4fe5b65` — implement bounded SOCKS4a/5,
  SOCKS-IRC filtering composition, lifecycle ownership, registry promotion, and
  M069 planning/status updates.

## 1. Executive finding

M069 is closed. `socks` and `socksirc` are real control-plane-owned backends.
They use the M065 local listener/session seam, perform bounded SOCKS4a or SOCKS5
TCP CONNECT negotiation, route direct I2P names through Yosemite/address-book
resolution, and use one explicitly configured I2P-hosted SOCKS5 outproxy for
clearnet targets. Literal IPs, local targets, unsupported commands, ambiguous
options, and unsafe exposure/authentication configurations fail closed before
listener/session allocation where applicable.

`socksirc` shares the exact M069 negotiation and target path with `socks` and
selects M066's stateful `relay_client_stream` for both payload directions. It
does not have a generic raw relay branch; DCC and unsupported CTCP therefore
retain M066's fail-closed behavior.

The public Proposal 170 contract remains partial because `httpbidirserver` and
both Streamr types remain unsupported, and the accepted RouterInfo 37/1/5
source limitation remains independent.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| I2PControl-only production scope | `backends/socks.rs`, `backends/socks_irc.rs`, registry/options/production changes | pass | No core, manifest, startup proxy, or dependency changes |
| SOCKS4a CONNECT | bounded parser and `socks4a_domain_connect_is_bounded_and_ignores_userid` | pass | Domain form only; USERID is never treated as identity |
| Plain SOCKS4 literal rejection | `socks4_literal_and_port_zero_fail_before_request` | pass | No local or remote allocation path |
| SOCKS4 bounds/termination | `BudgetedReader`, max 255-byte USERID/domain, 8 KiB total budget | pass | Missing terminators and overlong fields fail |
| SOCKS5 method negotiation | parser unit fixtures for no-auth and username/password | pass | Method count is bounded to 32 |
| Constant-time credential comparison | `i2pcontrol::auth::compare_passwords` | pass | Incoming username/password values are bounded and never logged |
| SOCKS5 username/password success/failure | `socks5_auth_success_and_failure_are_protocol_correct` | pass | RFC subnegotiation status is emitted before close on failure |
| CONNECT only | request command check and unsupported-command fixture | pass | BIND and UDP ASSOCIATE return `0x07` |
| Address-type safety | IPv4/IPv6 requests return `0x08`; only domain requests continue | pass | No literal target is passed to the OS |
| Port validation | zero-port request rejection in both protocol parsers | pass | Rejected before target routing |
| Negotiation bounds/time | `MAX_NEGOTIATION_BYTES`, field limits, `NEGOTIATION_TIMEOUT` | pass | Active connection cap is 128 per backend listener |
| Direct I2P routing | `target_route` + `resolve_destination` + `ClientStreamConnector::connect_to` | pass | No `ToSocketAddrs`, OS DNS, or direct TCP target connect |
| I2P aliases and B32 | approved runtime address-book lookup and `.b32.i2p` handling | pass | Missing alias authority fails closed |
| Literal/local/private safety | literal rejection plus shared HTTP target classifier for named local targets | pass | Loopback, private, link-local, multicast/unspecified literals cannot route |
| Explicit clearnet outproxy | `ProxyList` parsing requires an I2P destination and SOCKS/SOCKS5 type | pass | Multiple/unimplemented outproxy selection is rejected |
| Outproxy routing | bounded SOCKS5 handshake to the Yosemite-connected I2P outproxy | pass | Target hostname/port is conveyed only inside that I2P hop |
| Success reply ordering | handler connects direct/outproxy before `send_success` | pass | Connection failure returns protocol failure and no success |
| Neutral bind reply | SOCKS4 `0.0.0.0:0` and SOCKS5 IPv4 `0.0.0.0:0` | pass | No local routable interface is exposed |
| Exposure/authentication | loopback default; non-loopback or configured credentials require auth | pass | Incomplete credentials reject before reservation |
| Raw SOCKS payload boundary | `PayloadMode::Raw` uses bidirectional byte relay after success | pass | Documented as application-risk boundary |
| SOCKS-IRC composition | `PayloadMode::Irc` calls `relay_client_stream`; static guard test | pass | No raw `copy_bidirectional` in `socks_irc.rs` |
| IRC DCC/CTCP policy | M066 common filter and M069 static composition guard | pass | No duplicated command table or DCC path |
| Option capability | `SOCKS_OPTIONS`/`SOCKS_IRC_OPTIONS` plus raw-option allowlist | pass | I2CP/custom, BIND/UDP/DNS extensions, and unknown options reject |
| Lifecycle | shared generation supervisor and `both_payload_modes_start_stop_and_restart_with_exact_generation_cleanup` | pass | Duplicate start, stop, restart, cancellation, and stale generation handling bounded |
| Registry/startup promotion | production registry and `reconcile_start_on_load` include only the two new types | pass | Existing startup SOCKS remains untouched |
| Documentation/support state | `docs/i2pcontrol/proposal-170-support.md` | pass | Raw SOCKS anonymity limitation and unsupported surface documented |
| Successor planning | registry/roadmap/README updated; M070 ready, M071 unregistered-ready, M072 blocked | pass | M051 independent blocker unchanged |

## 3. Production implementation evidence

Changed production paths:

- `emissary-cli/src/i2pcontrol/backends/socks.rs` — shared parser,
  negotiation replies, target routing, I2P SOCKS5 outproxy adapter, raw/IRC
  payload modes, and generation-safe supervisor;
- `emissary-cli/src/i2pcontrol/backends/socks_irc.rs` — `socksirc` backend
  selecting the filtered payload mode;
- `emissary-cli/src/i2pcontrol/backends/options.rs` — typed capability matrices;
- `emissary-cli/src/i2pcontrol/backends/registry.rs` — only `socks` and
  `socksirc` replace unsupported registrations;
- `emissary-cli/src/i2pcontrol/production.rs` — StartOnLoad reconciliation for
  the two control-plane-owned types.

Containment authority was updated in
`emissary-cli/tests/m062_dependency_containment.rs` to authorize exactly the
two new I2PControl backend files and this closure record; no broader source or
dependency path was opened.

No startup listener/task was adopted. No `emissary-core/**` production path,
public JSON-RPC schema, persistence schema, action, or tunnel type was added.

## 4. Verification executed

### Commands run

```text
rustfmt +nightly --check --edition 2021 --config-path rustfmt.toml emissary-cli/src/i2pcontrol/backends/socks.rs emissary-cli/src/i2pcontrol/backends/socks_irc.rs
cargo test -p emissary-cli --no-default-features --features i2pcontrol socks --lib -- --nocapture --test-threads=1
cargo test -p emissary-cli --no-default-features --features i2pcontrol socks_irc --lib -- --nocapture --test-threads=1
cargo test -p emissary-cli --no-default-features --features i2pcontrol irc --lib -- --nocapture --test-threads=1
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment -- --nocapture --test-threads=1
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment -- --nocapture --test-threads=1
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib -- --nocapture --test-threads=1
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --lib -- -D warnings
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo check -p emissary-core --no-default-features
git diff --check
```

### Results

- Nightly rustfmt for the two new Rust files: pass.
- Focused M069 suite: 12 passed.
- M066 IRC-focused suite: pass; M061 containment: 7 passed; M062 dependency
  containment: 19 passed.
- Feature-enabled I2PControl library suite: pass; feature-enabled package
  check and default/no-feature CLI check: pass.
- Feature-enabled library Clippy with `-D warnings`: pass.
- Feature-enabled all-targets Clippy remains blocked by the inherited,
  unchanged startup-owned `emissary-cli/src/proxy/socks.rs` warning at line
  543 (`to_string` inside `format!`). This is outside M069's authorized path.
- `emissary-core --no-default-features` retains the inherited unrelated
  `RwLock` import failures in core session/inspection/profile/subsystem paths.
  M069 makes no core change and does not consume this broken configuration.
- Stable `cargo fmt --all -- --check` is not a valid repository check in this
  checkout because it rewrites widespread pre-existing code under the nightly-
  only rustfmt configuration. The scoped nightly check above is the accepted
  substitute used by the preceding M068 closure.
- `git diff --check`: pass.

## 5. Invariant review

All M069 hard invariants pass. Direct I2P requests do not invoke local DNS or
local TCP; literal IP and local target inputs fail closed; only explicit I2P
outproxy behavior can carry clearnet hostnames; non-loopback listeners require
credentials; secrets are redacted; BIND/UDP and unsupported options reject
before resource allocation; SOCKS-IRC calls the M066 filter; and startup-owned
SOCKS resources remain outside the backend.

The registry remains exhaustive. The default registry still maps all types to
resource-free unsupported backends, while only the two M069 production entries
are promoted. No default-build dependency or source-boundary expansion occurred.

## 6. Failure and recovery review

Malformed, overlong, unterminated, unsupported, unauthenticated, zero-port,
literal-address, local-target, missing-address-book, outproxy, and remote-connect
failures close only the accepted local connection and emit a protocol failure
where the request version is known. Success is emitted only after the Yosemite
stream and, when applicable, the outproxy SOCKS5 CONNECT succeed.

The supervisor reserves one exact name/generation, rejects duplicate starts,
drains bounded listener tasks on stop, removes the generation after stop, and
prevents stale task completion from changing a newer generation. The shared
M065 task group bounds active connections at 128 and runtime tasks at 1000.
Cancellation covers pending negotiations and active relays through listener
task cancellation and bounded task drain. Restart is stop followed by a fresh
session/listener generation.

## 7. Migration and compatibility review

There is no wire, persistence, or migration schema change. Existing persisted
`socks`/`socksirc` definitions become startable only when their typed and raw
options fit the explicit implemented capability matrix. Unknown recognized
security/runtime options reject instead of being accepted and ignored. The
startup SOCKS service and its ownership remain unchanged. SOCKS payload bytes
remain intentionally opaque in the normal `socks` type; users requiring the
M066 IRC safety boundary must select `socksirc`.

## 8. Security review

Authentication uses the existing constant-time comparison helper and bounded
credentials. Debug output includes usernames only and replaces password values
with redacted markers; parser errors contain only option/status labels. Local
DNS, direct clearnet, localhost/LAN routing, literal target routing, open
non-loopback unauthenticated exposure, BIND, UDP, and arbitrary resolve
extensions are unavailable. Connection and negotiation resources are bounded,
and the filter path cannot be bypassed for SOCKS-IRC.

## 9. Documentation and operations

Support documentation now states the exact SOCKS4a/SOCKS5 command/address
surface, authentication/exposure rules, I2P outproxy requirement, and the raw
application-protocol anonymity limitation. The active registry, tunnel-runtime
roadmap, implementation handoff README, and M069 plan now point to this closure
and promote M070 as the next registered handoff.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low / inherited | Feature-enabled all-targets Clippy fails on unchanged startup `proxy/socks.rs` line 543 | Does not affect I2PControl M069 paths | Preserve as existing repository limitation; do not widen M069 scope |
| low / inherited | Core no-default check fails on unrelated missing `RwLock` imports | M069 has no core changes and no core runtime dependency | Preserve as existing baseline limitation; address under a separate core plan if assigned |

No high- or medium-severity M069 finding remains open.

## 11. Roadmap disposition

Milestone closed and next dependency may proceed. M070 is now the single
registered `ready` successor because M067, M068, and M069 are closed. M071 is
also dependency-ready from M065 but remains unregistered until M070 is handled,
following the planning-process rule. M072 remains blocked until M070 and M071
close. Independent M051 remains blocked by absent substantive RouterInfo
news/ban owners and is unaffected.

## 12. Registry updates

Applied in this change:

- M069 plan and closure are `closed`;
- the tunnel-runtime roadmap is `active; M064-M069 closed; M070 is the next
  registered handoff`;
- M070 is the dependency-ready registry handoff;
- M071 is named dependency-ready but unregistered;
- M072 remains blocked on M066-M071.

Internal-only attestation: external specifications and reference material, if
consulted, were read-only. No upstream repository, issue, pull request,
review, merge, adoption, submission, contribution package, or maintainer
channel was mutated or prepared. Repository writes remained within the
authorized internal `eggstack/emissary` repository.

Final disposition: **closed**.
