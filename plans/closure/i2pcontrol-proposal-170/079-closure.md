# M079 Closure — Proposal 170 Tunnel Security Reclosure

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/079-tunnel-security-reclosure.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`

Planning baseline:

- `04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`

## 1. Disposition

M079 independently reclosed the actual final head after M073-M078. The
integrated twelve-type tunnel runtime passes the required resource, timing,
identity-boundary, lifecycle, option-truthfulness, containment, and local
fixture evidence. Two final-head defects found during this review were fixed
inside the authorized I2PControl boundary:

- persisted public server destinations are now identity metadata only and
  cannot become an HTTP/httpbidir local target on restart;
- arbitrary `X-I2P-*` request headers are removed alongside the named identity
  headers, and malformed raw HTTP string options fail before allocation.

Admission also checks aggregate-rate availability before allocating a new peer
record. This prevents aggregate-denied identity churn from consuming bounded
peer-state slots.

M079 closes with no unresolved high- or medium-severity security, anonymity,
correctness, lifecycle, or containment finding. Proposal 170 remains
`partial Proposal 170 support` because the unrelated RouterInfo 37/1/5 source
disposition is unchanged.

## 2. M073-M078 implementation range

The exact corrective-series commits reviewed were:

| Milestone | Implementation evidence |
|---|---|
| M073 | `3d1d8f1232a7a25dbff72ef81d63886c7b90bf75` — generic option truthfulness and shared admission implementation head |
| M074 | `3d1d8f1232a7a25dbff72ef81d63886c7b90bf75` — server admission implementation; `a17389b5734629b8103436fb3550f22294c5c0d9` closure |
| M075 | `20db126325c858b6a240d49f4bdbe436ab184a50` — generic accepted-stream server |
| M076 | `3cf082ef19efd490db6e5c602bebd5e0e95207cb` and `f454e3541bf51df57b6870b0bf0a324e5655266f` — HTTP anonymity and forwarded-header corrections |
| M077 | `0660ca6120ec378142b2230121130deac890dbec` — IRC server lifetime hardening |
| M078 | `0ff8b221d1fc56b70c6fbc22d2cc1cdd72c9efa0` — Streamr local-boundary hardening |

The preceding M073-M078 closure records were independently checked at the
final integrated head; M079 does not accept their assertions without rerun or
source reconciliation.

## 3. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| All twelve production tunnel types reviewed | Backend registry, production composition, specialized backend source, focused/full test suites | pass |
| One peer cannot monopolize global accepted capacity | `ServerAdmissionState::try_acquire`; peer-concurrency and global-limit tests | pass |
| Distinct peers eventually hit aggregate limits without unbounded state growth | Paused aggregate-window test; new peer state is not allocated on aggregate denial; fixed `MAX_PEER_ENTRIES` | pass |
| Per-peer/hour/day windows cross boundaries correctly | Paused admission window tests | pass |
| Full limiter table fails closed without active-state eviction | Full peer-table test; no eviction path; expired state is reclaimed only by monotonic expiry | pass |
| Lease/task/state release covers error, panic, cancel, and stop | RAII `AdmissionLease`, panic isolation, `BoundedTaskGroup`, generation-specific stop tests | pass |
| Generic `server` uses accepted-stream admission | `run_accepted_server` composition and generic raw-relay fake-SAM capture test | pass |
| Generic bytes remain raw after admission | `generic_server_uses_accepted_stream_and_relays_bytes_without_forwarding` | pass |
| IRC registered idle expires and active traffic survives | Paused-time `irc_server` relay tests; ten-minute inactivity deadline resets in either direction | pass |
| HTTP request proxy/I2P identity spoofing is removed | Table-driven HTTP filter tests, including arbitrary mixed-case `X-I2P-*` | pass |
| Trusted identity injection is bounded | Accepted-stream identity bound 524 bytes; HTTP destination validation bound 524 bytes; malformed identity rejection | pass |
| HTTP response `Date`, `Server`, provider/cache/trace fingerprints are removed | Mixed-case response denylist tests and local response capture | pass |
| HTTP framing stays unambiguous | Conflicting lengths/transfer encoding rejection; normalized Content-Length/chunked tests | pass |
| POST limiter is bounded and churn-safe | Paused table-saturation, expiry, independent-peer, and rejected-before-connect tests | pass |
| `httpbidirserver` uses the exact inbound HTTP filter/admission path | Shared `make_accepted_handler`/`PostLimiter` composition and lifecycle tests | pass |
| IRC registration remains fail-closed and peer-derived | Registration bounds, wrong-protocol, malformed input, trusted USER rewrite, and target ordering tests | pass |
| Streamr local producer/client boundaries are loopback-only | Address validation tests for defaults, v4/v6 loopback, unspecified/LAN/public/non-loopback values | pass |
| Streamr fanout/control/payload bounds remain exact | Ten-subscriber ceiling, no-eviction refresh, 60-second expiry, 15-second refresh, one-byte controls, 1200/4095 bounds | pass |
| Remote Streamr payload cannot select a local destination | Fixed configuration target and local-target tests | pass |
| Persistent server identity is stable; ephemeral state is not persisted | Server/Streamr lifecycle tests and backend-owned destination store | pass |
| Published server destination cannot become a local target | New HTTP/httpbidir restart-metadata regression tests | pass |
| Failed definitions do not fail unrelated StartOnLoad reconciliation | Production manager isolated reconciliation path and retained lifecycle tests | pass |

## 4. Threat-model and anonymity review

### Resource exhaustion and fairness

Accepted server handlers have one global ceiling, a fixed per-peer ceiling,
peer connection windows, aggregate windows, and a hard peer-state table bound.
Admission state is generation-local and uses RAII leases. The aggregate check
now precedes insertion of a new identity, while table-full behavior remains
fail-closed. No active or throttled state is evicted to admit attacker-
controlled identities. HTTP write quotas use the same bounded fail-closed
pattern. IRC registered-idle streams have a deterministic ten-minute
inactivity bound. Streamr uses one owner loop, ten subscribers, sequential
fanout, fixed buffers, and no per-packet task queue.

### Timing and correlation

No single peer can reliably toggle the global capacity state because peer
concurrency and rate admission are checked before handler allocation. Overload
is rejected promptly; no artificial sleep, jitter, or padding defense was
introduced. Local target failures are reduced to static protocol errors or a
quiet close and do not disclose target/OS details. Application response
latency, congestion, and ordinary I2P path timing remain unavoidable residual
risk; M079 does not claim constant-time networking or public-network
deanonymization protection.

### HTTP identity/fingerprint boundary

The request filter removes the complete `X-Forwarded-*` namespace, named
proxy identity headers, and the complete `X-I2P-*` namespace before rebuilding
only trusted peer-derived identity headers. The accepted-stream identity is
ASCII, whitespace/control-free, and capped at the reference-sized 524-byte
representation. Response `Date`, `Server`, adopted provider/cache/trace
fingerprints, and hop-by-hop headers are stripped without rewriting the
application body or changing validated framing. POST accounting happens before
the local target connect and remains bounded under identity churn.

### IRC boundary

Registration filtering precedes local connect. The hostname is derived only
from the SAM-accepted peer destination; malformed, incomplete, and wrong-
protocol registrations fail closed. The ten-minute timeout is activity-resetting
and does not cap active total lifetime. DCC, WEBIRC, and unsupported CTCP paths
remain explicit unsupported behavior with no auxiliary bypass.

### Streamr boundary

Both local UDP endpoints are loopback-only and reject non-loopback configuration
before session/socket allocation. Remote packets never select the client local
target. Subscriber identity, control packet shape, expiry, refresh cadence,
payload size, transport receive buffer, sequential fanout, and shutdown bounds
remain finite and reference-aligned.

## 5. Option-capability matrix disposition

The final matrix is recorded in `docs/i2pcontrol/tunnel-backends.md`. In
summary, every runtime-relevant field is consumed or rejected before
allocation:

| Tunnel type | Applied/runtime-owned fields | Rejected before allocation |
|---|---|---|
| `client` | Destination, target/listener ports, listener interface | Unsupported typed/raw, access/plaintext, custom/I2CP |
| `httpclient` | Listener/auth, HTTP policy, direct-I2P target, explicit I2P outproxy | TLS, unsafe/direct clearnet, unsupported proxy/outproxy, custom/I2CP |
| `ircclient` | I2P destination/ports, listener, common IRC filter | IRC automation, WEBIRC/cloak/access/auth, custom/I2CP |
| `socks` | Authenticated/loopback listener, bounded SOCKS CONNECT and target policy | BIND, UDP ASSOCIATE, arbitrary DNS/unsafe targets, custom/I2CP |
| `socksirc` | SOCKS runtime plus common IRC filter | SOCKS/IRC unsupported paths, custom/I2CP |
| `connectclient` | Strict CONNECT listener/auth, direct-I2P or explicit I2P outproxy | Unsupported methods/targets/modes, custom/I2CP |
| `streamrclient` | Producer destination, loopback target, ports, refresh | Non-loopback and unsupported shaping/signature/encryption/custom/I2CP |
| `server` | Loopback target/port, persistent identity, shared admission, `leaseSetEncType` | Unsupported privacy/access/consumer/signature/hashcash/raw/custom/I2CP |
| `httpserver` | Loopback target, Host/access policy, admission, POST limiter, identity | TLS/proxy/outproxy/filter/address/period options, custom/I2CP |
| `httpbidirserver` | Shared HTTP inbound path, local proxy, loopback bind/target, admission/POST limiter | Unsupported TLS/outproxy/filter/address/period options, custom/I2CP |
| `ircserver` | Registration filter, trusted hostname, loopback target, admission, inactivity relay | IRC automation/WEBIRC/cloak/access/auth/DCC, custom/I2CP |
| `streamrserver` | Persistent identity, loopback UDP source, ten subscribers, exact datagram bounds | Non-loopback and unsupported shaping/signature/encryption/custom/I2CP |

`HostingDestination` is published server identity metadata and is not a local
target selector. `PerClientPeriod`, `TotalPeriod`, `TotalBanTime`,
`FilterFilePath`, `UniqueLocalAddressPerClient`, `MultiHoming`, and other
recognized-but-unimplemented fields remain explicit rejections where their
semantics are not owned by the backend.

## 6. Lifecycle, restart, and contention review

All real backends validate typed/raw options before store lookup, listener,
session, socket, or task allocation. Runtime readiness is published only after
the underlying session and local owner are established. Stop is idempotent;
restart uses complete stop followed by a new generation. Generation checks
prevent an old task from mutating a replacement entry, and bounded task groups
drain or abort within the declared stop timeout.

Persistent server identities remain in the backend-owned secret store and
public destinations remain stable across restart. Ephemeral admission,
limiter, Streamr subscriber, and runtime task state is generation-local. The
HTTP/httpbidir published-destination regression test verifies that persisted
public identity metadata cannot select a local target. One failed definition
is isolated during StartOnLoad reconciliation.

## 7. Containment and dependency diff review

Compared with baseline `04e0c2e`, all runtime production changes remain under
`emissary-cli/src/i2pcontrol/**`; no `emissary-core/**` production path,
startup-service ownership refactor, hosted CI/release/fuzz/soak infrastructure,
or public Proposal 170 field was added. The only manifest adjustment in the
series is test-local: Tokio `test-util` is now in `dev-dependencies`, not the
normal dependency, so default/feature-disabled production builds do not gain
test-only functionality. No new default-enabled dependency was introduced;
the I2PControl-only `subtle` dependency remains optional and feature-owned.

M061 source containment and the M062/M063 dependency-feature authorities were
rerun. Their allowlist accepts only the scoped I2PControl source, test, docs,
and planning evidence. No unrelated source or dependency path is present in
the final diff.

## 8. Verification commands and outcomes

Passed:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment
rustfmt +nightly --check --edition 2021 --config-path rustfmt.toml \
  emissary-cli/src/i2pcontrol/backends/runtime/admission.rs \
  emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs \
  emissary-cli/src/i2pcontrol/backends/filters/http.rs \
  emissary-cli/src/i2pcontrol/backends/http_server.rs \
  emissary-cli/src/i2pcontrol/backends/http_bidir.rs
git diff --check
```

The full feature-enabled CLI suite passed 1,594 tests across 24 suites.
The focused M061/M062-M063 containment command passed 26 tests (7 source
containment and 19 dependency containment). Both CLI checks, the core check,
strict Clippy, scoped nightly rustfmt, and `git diff --check` passed.

The required stable `cargo fmt --all -- --check` was run and remains red on
inherited repository-wide formatting drift because `rustfmt.toml` uses
nightly-only options. It also reports unrelated workspace files outside this
scope; no unrelated formatter changes were retained. The touched Rust files
pass the repository-accepted scoped nightly formatter check.

## 9. Unresolved findings

| Severity | Finding | Disposition |
|---|---|---|
| low/environmental | Stable repository-wide rustfmt cannot honor the configured nightly-only options and reports inherited unrelated drift | Documented limitation; scoped nightly rustfmt passes; no runtime action required |
| — | No high/medium security, anonymity, correctness, lifecycle, or containment finding | M079 acceptance criterion satisfied |

## 10. Documentation and planning disposition

Updated:

- `docs/i2pcontrol/proposal-170-support.md`;
- `docs/i2pcontrol/tunnel-manager.md`;
- `docs/i2pcontrol/tunnel-backends.md` with the twelve-type capability matrix;
- `plans/implementation/i2pcontrol-proposal-170/079-tunnel-security-reclosure.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/registry.md`;
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`.

M079 is closed, the security-hardening roadmap is closed, and no future
tunnel-security plan is dependency-blocked and newly unblocked. M051 remains
independently blocked by the accepted absence of substantive RouterInfo
news/banned-peer owners. No later security successor is prewritten or
registered ready.

## 11. External read-only research attestation

The pinned Proposal 170 contract and Java/I2P+ behavioral references were
consulted only as read-only evidence. The public contract reference was
`https://i2p.net/en/proposals/170-i2pcontrol-expansion/`; the Java reference
repository was `https://github.com/i2p/i2p.i2p`. No public-network experiment,
deanonymization experiment, or external interoperability certification was
performed.

No upstream issue, pull request, review, submission, maintainer contact,
adoption request, merge, or repository write occurred. No contribution
artifact was prepared. All implementation, documentation, planning, commit,
and push activity is limited to the authorized internal `eggstack/emissary`
repository.
