# M078 Closure — Streamr Local Boundary Hardening

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/078-streamr-local-boundary-hardening.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`

Repository baseline reviewed: `682a960`

Implementation commit:

- `0ff8b22` — feat(i2pcontrol): harden Streamr local boundary

## 1. Executive finding

M078 is closed. Streamr local UDP ingress and client output are now loopback-only,
non-loopback requests fail during backend configuration before supervisor/session/
socket allocation, subscriber fanout is aligned to the Java reference ceiling of
10, and the existing expiry, refresh, payload, transport, control, sequential
fanout, cancellation, and persistent-identity boundaries remain intact.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| Default local addresses are loopback | `DEFAULT_BIND_ADDRESS`; `loopback_defaults_and_explicit_v4_v6_addresses_are_accepted` | pass | Defaults to `127.0.0.1`. |
| Explicit `127.0.0.1` and `::1` are accepted | `local_loopback_address`; loopback address test | pass | Both client target and server bind paths are covered. |
| Non-loopback local addresses fail before allocation | `local_loopback_address`; `non_loopback_addresses_reject_before_runtime_reservation` | pass | Covers unspecified, LAN/documentation, public/documentation, and non-loopback IPv6 values plus typed `ListenInterface`. |
| No silent coercion or value disclosure | `non_loopback_local_address` and sanitized parse errors | pass | Errors name the option and loopback/IP policy without echoing the configured value. |
| Unexpected non-loopback UDP source is dropped | `local_udp_source_allowed`; server receive loop | pass | Defense-in-depth check precedes payload fanout. |
| Subscriber maximum is 10 with no eviction | `MAX_SUBSCRIBERS`; `subscriptions_are_bounded_and_refresh_in_place` | pass | The eleventh destination is rejected and existing state remains. |
| Existing subscriber refresh works at capacity | `SubscriptionState::apply_control` and capacity test | pass | Refresh updates the timestamp without changing set size. |
| Expiry remains exactly 60 seconds | `SUBSCRIPTION_EXPIRY`; expiry test; paused-time-capable monotonic `Instant` logic | pass | Expiry uses monotonic time and removes at the boundary. |
| Client refresh remains exactly 15 seconds | `SUBSCRIPTION_REFRESH`; exact-bounds test | pass | Existing interval and delayed-tick behavior preserved. |
| Payload and transport bounds remain 1200/4095 bytes | `payload_is_forwardable`; exact-bounds test | pass | Oversized payloads are dropped before local send/fanout. |
| Destination/control input is bounded | `MAX_STREAMR_DESTINATION_TEXT`; `valid_destination`; `apply_control` tests | pass | Text is capped at 524 bytes; controls remain exactly one byte and unknown/malformed controls create no state. |
| All-ten identity text memory is documented | `docs/i2pcontrol/streamr-runtime.md` | pass | At most 5,240 bytes of destination text, plus fixed map/storage overhead. |
| Remote payload cannot select local target | Fixed `StreamrClientConfig.local_target`; `local_target_is_fixed_by_configuration`; no payload-derived target path | pass | Target is selected only during pre-allocation configuration. |
| No unbounded task/queue/fanout path | Existing owner loop, snapshot, sequential sends, and unchanged runtime bounds | pass | No per-packet task or send queue was introduced. |
| Restart clears ephemeral subscribers and retains server identity | M071 closure restart evidence and unchanged `DestinationKind::Persistent`/secret-store owner | pass | M078 does not alter lifecycle or persistence ownership. |
| Production scope remains I2PControl-local | `0ff8b22` diff and M062 containment suite | pass | Runtime changes remain under `emissary-cli/src/i2pcontrol/**`; docs and a containment allowlist are the only other changes. |

## 3. Production implementation evidence

`local_loopback_address` validates every supplied `TargetHost`, `Host`,
`ReachableBy`, and typed `ListenInterface` value rather than selecting one and
silently ignoring the others. The client binds its ephemeral UDP output socket
to the validated loopback family. The server binds only the validated loopback
address and checks the observed UDP source before applying the existing payload
cap or snapshotting subscribers.

The subscriber map now uses the 524-byte destination-text bound and a ten-entry
ceiling. It still stores the complete trusted destination needed by Yosemite,
uses monotonic `Instant` timestamps, refreshes in place, expires without
per-entry timers, snapshots before network I/O, and performs sequential sends.
The persistent server destination and generation-safe supervisor are unchanged.

The M062 dependency-containment guard was extended to recognize the already
landed M077 closure and this M078 closure. This is planning-test bookkeeping,
not a production dependency or runtime expansion.

## 4. Verification executed

### Commands run

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol streamr
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
rustfmt +nightly --check --edition 2021 --config-path rustfmt.toml emissary-cli/src/i2pcontrol/backends/streamr.rs
cargo fmt --all -- --check
git diff --check
```

### Results

The focused Streamr run passed 20 tests. The complete feature-enabled CLI
suite passed 1,586 tests across 24 suites. Both CLI checks, the core check,
strict Clippy, M061 containment (7 tests), M062 dependency containment (19
tests), and scoped nightly rustfmt passed. `git diff --check` passed before the
closure-only planning edits.

The repository-wide stable `cargo fmt --all -- --check` command remains an
inherited environmental/toolchain limitation: the repository configuration
uses nightly-only rustfmt options and stable rustfmt reports unrelated diffs
across the existing workspace. It was not used to rewrite unrelated files;
the touched Rust file passes the repository's nightly formatter check.

## 5. Invariant review

- Streamr remains a dedicated I2PControl datagram runtime; no core/router UDP
  API or generalized UDP framework was added.
- Local producer ingress and client output are loopback-only, with truthful
  rejection of non-loopback configuration and no local authentication field.
- Remote datagrams retain authenticated destination identity as the subscriber
  key and never select or rewrite the client local target.
- The subscriber set, identity text, receive buffers, payloads, runtime tasks,
  sequential fanout, expiry scan, and shutdown attempt remain bounded.
- Cancellation/restart continues to use the existing supervisor generation;
  subscriber state is ephemeral and the persistent server destination remains
  backend-owned and stable.

## 6. Failure and recovery review

Malformed, unknown, whitespace/control-containing, slash-containing, oversized,
and absent-peer controls are rejected without state creation. At capacity a new
destination is rejected without eviction; an existing destination can still
refresh. Oversized local UDP payloads and unexpected non-loopback local sources
are dropped. Configuration errors return before supervisor reservation, so they
cannot create a SAM session, socket, task, or runtime entry. Existing stop,
timeout, panic-isolation, and generation cleanup behavior is unchanged.

## 7. Migration and compatibility review

No persistent schema, Proposal 170 field, control packet, session-port tuple,
Yosemite API, or router-core behavior changed. The accepted local-address
surface is intentionally narrowed: non-loopback configurations that previously
started now fail explicitly and must be changed to loopback. Valid loopback
IPv4/IPv6 configurations, base32-sized identities, ordinary destination text,
and the established one-byte control protocol remain supported.

## 8. Security review

The former unauthenticated LAN/external UDP injection and remote-triggered
external local-target risks are closed by validation before allocation and by
binding the client output socket to loopback. The source-address check adds
defense in depth without a new same-host authentication protocol. The ten-entry
ceiling reduces fanout amplification to the reference scale, while the 524-byte
identity bound removes the prior arbitrary 64 KiB remote identity allocation.
No high- or medium-severity Streamr integrity, anonymity, resource, lifecycle,
or input-boundary finding remains in M078 scope.

## 9. Documentation and operations

Updated:

- `docs/i2pcontrol/README.md`;
- `docs/i2pcontrol/proposal-170-support.md`;
- `docs/i2pcontrol/streamr-runtime.md`;
- `docs/i2pcontrol/tunnel-manager.md`;
- M078 implementation status, registry, and tunnel-security roadmap;
- M062 containment allowlist for the M077/M078 closure records.

The active registry now records M079 as the single ready successor.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low/environmental | Stable repository-wide rustfmt cannot honor the configured nightly-only options | Formatting check remains unavailable for untouched workspace files | Continue using the repository-accepted nightly scoped formatter; no M078 runtime action. |
| — | No M078 production finding | — | — |

## 11. Roadmap disposition

Milestone closed and the next dependency may proceed. M079 is unblocked, marked
`ready`, and registered as the sole dependency-ready handoff for integrated
tunnel-security reclosure. M051 remains independently blocked by its accepted
RouterInfo source limitation and is unaffected by M078.

## 12. Registry updates

Updated `plans/registry.md`,
`plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`,
`plans/implementation/i2pcontrol-proposal-170/README.md`, M078's status, and
M079's status. Added this closure record. M078 is closed; M079 is ready.

## 13. Internal-only external interaction attestation

Java I2P and Proposal 170/reference material were used only as read-only
behavioral evidence. No upstream repository, maintainer channel, issue, pull
request, review, merge, adoption, or submission was mutated or requested. The
implementation and planning commits are internal changes pushed only to the
authorized `eggstack/emissary` fork remote.
