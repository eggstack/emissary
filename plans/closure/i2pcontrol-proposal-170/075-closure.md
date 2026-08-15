# M075 Closure — Generic Server Accepted-Stream Hardening

Status: closed against implementation commit `20db126`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/075-generic-server-accepted-stream-hardening.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`

## 1. Disposition

M075 is closed. The control-plane generic `server` backend no longer uses
blind SAM `STREAM FORWARD`. It now owns an application-visible accepted-stream
session, reuses M074's peer-aware admission state, and performs a raw relay
only after admission to the fixed loopback target.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Generic control-plane server uses accepted-stream operations | `backends/server.rs::ServerRuntimeSupervisor::start`; `run_accepted_server` composition | pass |
| Generic control-plane path does not issue `STREAM FORWARD` | `generic_server_uses_accepted_stream_and_relays_bytes_without_forwarding` fake-SAM test | pass |
| M074 admission is reused before handler/target work | `AcceptedServerRuntimeConfig` receives `ServerAdmissionPolicy`; shared `run_accepted_server` ordering | pass |
| Raw bytes reach the fixed local target in both directions | fake-SAM/TCP fixture relays `from-i2p` and `to-i2p` byte-for-byte | pass |
| Loopback target confinement and bounded connect | `runtime_config` accepts only `127.0.0.1`/`localhost`; relay uses a five-second timeout and normalized `127.0.0.1` | pass |
| Denied/failed streams do not allocate a target through the generic handler | shared accepted-server admission precedes handler; relay returns before target connect failure can affect the session | pass |
| Generic admission controls are applied before store/session allocation | raw allowlist plus `ServerAdmissionPolicy::from_raw_options`; admission/loopback regression test | pass |
| Unsupported generic options remain explicit rejects | existing typed/raw option tests remain green; `AccessList`, filter, TLS, multi-home, and underspecified period fields remain rejected | pass |
| Persistent destination/public identity is stable across restart | generic lifecycle test stops and starts the same definition and compares published destination | pass |
| Exact-generation cancellation and bounded child draining | existing per-name supervisor generation/cancellation plus shared accepted-server `BoundedTaskGroup`; lifecycle test remains green | pass |
| Child relay errors cannot kill unrelated accepted streams | shared accepted-server task panic/error isolation and M074 runtime tests | pass |
| Private destination material is secret-safe | `StoredDestination` redaction plus `generic_server_debug_is_secret_safe`; sanitized runtime errors | pass |
| Startup-managed server ownership/path is unchanged | implementation diff is confined to `emissary-cli/src/i2pcontrol/**`; startup forwarding remains in `emissary-cli/src/tunnel/server.rs` | pass |
| M061/M062/M063 containment remains intact | `m061_containment`: 7 passed; `m062_dependency_containment`: 19 passed | pass |

## 3. Implementation details

The generic supervisor now starts `AcceptedServerRuntimeConfig` with the
backend-owned `StoredDestination`, configured admission policy, and a handler
that connects only to `127.0.0.1:<target-port>`. The accepted runtime reports
the actual public destination after Yosemite session creation; the supervisor
publishes it only for the matching generation.

The seven M074 connection/admission fields are accepted only in the generic
backend's bounded raw allowlist and are parsed into the shared policy. Access
lists, protocol filters, TLS termination, arbitrary target routing,
`UniqueLocalAddressPerClient`, `MultiHoming`, and guessed period/ban semantics
remain explicit unsupported options.

No `emissary-core/**` production code, SAM protocol extension, startup server
path, durable identity format, public Proposal 170 field, or application
protocol parser changed.

## 4. Verification

Passed:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
rustfmt +nightly --check --edition 2021 --config-path rustfmt.toml emissary-cli/src/i2pcontrol/backends/server.rs
git diff --check
```

The package suite passed 1,546 tests across 24 suites. The focused generic
server suite, including accepted-SAM, relay, restart, admission, startup
ownership, and secret-redaction coverage, is included in that package result.

The repository-wide stable `cargo fmt --all -- --check` remains red because
the repository's nightly-only rustfmt configuration and inherited formatting
drift affect unrelated files. The plan-scoped nightly check passed.

The requested feature-disabled core check,
`cargo check -p emissary-core --no-default-features`, remains blocked by the
pre-existing missing `RwLock` imports in unrelated core modules (`destination`,
`inspection`, `profile`, `subsystem`, `tunnel`, and `router` paths). The
required default core check passed, and M075 made no core change.

## 5. Invariant, compatibility, and security review

- Destination identity remains backend-owned, persistent, and absent from
  diagnostics.
- Remote identity is accepted only from Yosemite's accepted stream and is
  passed to M074 admission before handler execution.
- Local target selection is administrator-configured, fixed at start, and
  loopback-only; remote data never selects it.
- No lock is held across local target connection or relay.
- Generic payloads remain uninterpreted after admission; no HTTP, IRC, SOCKS,
  or other application semantics were introduced.
- Cancellation drains accepted child tasks through the existing bounded
  accepted-server runtime and cannot let an old generation update a new one.
- Startup-managed server forwarding remains outside TunnelManager ownership.
- No high/medium finding remains in the M075 generic-server scope.

## 6. Documentation and future-plan disposition

Updated:

- `docs/i2pcontrol/tunnel-backends.md`;
- `docs/i2pcontrol/tunnel-manager.md`;
- `docs/i2pcontrol/proposal-170-support.md`;
- the M075 handoff, implementation README, registries, and security/runtime
  roadmaps.

M072 is now recorded as closed after M073. M075 is closed. M076 is unblocked,
marked ready, and registered as the next dependency-ready handoff. M077 is
also hard-dependency ready but remains unregistered under the one-ready-plan
sequencing rule. M078 remains blocked by the ordered M075-M077 sequence, and
M079 remains blocked until M074-M078 close.

## 7. Internal-only external interaction attestation

External specifications and reference material were accessed read-only.
No upstream repository, maintainer channel, issue, pull request, review,
merge, adoption, or submission was mutated or requested. No upstream
contribution artifact was prepared under M075. The only push authorized for
this handoff is the internal repository remote `eggstack/emissary` requested
by the maintainer.
