# I2PControl Proposal 170 Milestone M065 — Closure Status

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/065-i2pcontrol-tunnel-runtime-primitives.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`

Planning production baseline: `a1296b018ce98d26a019bd5064dff9f4b47e0ad6`.

Implementation commit: this closing commit.

## 1. Executive finding

M065 closes the I2PControl-owned runtime and option-capability foundation. It
adds no new operational tunnel type and leaves the production registry mapped
to real generic `client`/`server` backends plus ten explicit unsupported
backends.

The client primitive owns a validated local listener, one independent
Yosemite streaming session, a narrow outbound-stream connector, and bounded
connection tasks. The accepted-server primitive owns a persistent Yosemite
session using a stored destination, exposes application-visible accepted
streams, and passes a SAM-derived public peer identity to a bounded handler
before any local target connection can be made by that handler.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Local listener readiness and bounded stop | `backends/runtime/client_listener.rs`; fake-SAM listener test reports the actual bound address and drains on cancellation | pass |
| One outbound session and narrow handler capability | `ClientStreamConnector` uses Yosemite detached stream creation and releases its mutex before await | pass |
| Accepted-stream interception without blind forwarding | `backends/runtime/accepted_server.rs` uses `Session::accept()` and never `forward()`; test handler receives the stream | pass |
| Trusted peer identity | `TrustedPeerIdentity` is constructed only from `Stream::remote_destination()` and exposes only the public destination | pass |
| Reject before local target connection | accepted-server test records identity, drops the stream, and asserts the target-connect marker remains false | pass |
| Task bounds and panic isolation | `BoundedTaskGroup` caps client/accepted handlers at 128, drains for 5 seconds, then aborts remaining tasks; panic test keeps listener alive | pass |
| Start failure cleanup | client/server setup-failure tests publish sanitized readiness errors and do not report a ready resource | pass |
| Option capability validation | `backends/options.rs`; required/optional, custom namespace, and security-option rejection tests | pass |
| Fail before allocation | generic client validates before runtime start; generic server validates before store lookup/session start | pass |
| Secret redaction | option/runtime errors contain field names only; stored destination keeps redacted `Debug`/`Display` | pass |
| Generation safety | existing generic supervisors retain per-name generations; stale `set_task` handles are aborted and stale completion checks generation | pass |
| Production registry containment | `backends/registry.rs` unchanged: only `client` and `server` are real in production | pass |
| Feature/default and source containment | M061/M062 tests plus feature-disabled checks | pass |

## 3. Exact changed paths

Production paths:

- `emissary-cli/src/i2pcontrol/backends/runtime/mod.rs`
- `emissary-cli/src/i2pcontrol/backends/runtime/task_group.rs`
- `emissary-cli/src/i2pcontrol/backends/runtime/client_listener.rs`
- `emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs`
- `emissary-cli/src/i2pcontrol/backends/options.rs`
- `emissary-cli/src/i2pcontrol/backends/mod.rs`
- `emissary-cli/src/i2pcontrol/backends/client.rs`
- `emissary-cli/src/i2pcontrol/backends/server.rs`

Documentation and planning paths are listed by the final commit. No core,
util, startup proxy, Cargo manifest, lockfile, CI, fuzz, release, or public
Proposal 170 schema path changed.

Documentation and planning paths:

- `AGENTS.md`
- `README.md`
- `docs/i2pcontrol/README.md`
- `docs/i2pcontrol/inspection-architecture.md`
- `docs/i2pcontrol/proposal-170-support.md`
- `docs/i2pcontrol/tunnel-manager.md`
- `plans/implementation/i2pcontrol-proposal-170/065-i2pcontrol-tunnel-runtime-primitives.md`
- `plans/implementation/i2pcontrol-proposal-170/README.md`
- `plans/registry.md`
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`
- `plans/closure/i2pcontrol-proposal-170/065-closure.md`

The M062 dependency-containment allowlist was extended for these exact M065
paths and the new runtime source files; no broader source exception was added.

## 4. Lifecycle, failure, and contention outcomes

- listener/session readiness is published only after both resources exist;
- bind or SAM setup failure returns a sanitized error and leaves no ready
  listener/session owner;
- handler panics are joined as per-connection failures and do not stop the
  instance;
- cancellation stops acceptance and drains exact current-generation child
  tasks within 5 seconds before aborting leftovers;
- task capacity is bounded to 128 connections per runtime instance;
- the existing generic supervisors reject duplicate active starts, make absent
  stops idempotent, and use monotonically changing per-name generations;
- stale completion cannot update a replacement generation, and a task that
  finishes before registration is not allowed to install a stale handle.

No lock is held across network I/O, sleeps, cancellation waits, or joins.

## 5. Verification executed

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol backends::runtime -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol backends::options -- --nocapture
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core --no-default-features
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo nextest run -p emissary-cli --no-default-features --features i2pcontrol --cargo-profile testnet
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --lib -- -D warnings
cargo fmt --all -- --check
git diff --check
```

The focused runtime and option suites pass in both library and binary test
targets, and the feature-enabled nextest run passed 1,420 tests. The exact
feature-disabled core check remains subject to the workspace's existing
feature-selection caveat recorded in M064; the CI-authoritative `no_std` and
`std` checks pass. Stable rustfmt may continue to report pre-existing unrelated
drift; no unrelated formatting cleanup is included here.

## 6. Containment and compatibility review

- Yosemite already supplies `Session::accept()` and
  `Stream::remote_destination()`, so no core API or neutral exception was
  needed.
- The ten specialized production mappings remain unsupported and resource
  free. M065 does not register runtime helpers as tunnel types.
- Startup-managed tunnel ownership and existing generic client/server owners
  remain unchanged.
- No persistence schema, wire field, action, tunnel type, dependency, lockfile,
  CI, fuzz, coverage, platform, or upstream artifact changed.
- Runtime option errors report only sanitized option identifiers and never raw
  request content or secret values.

## 7. Successor disposition

M066, M067, M068, and M071 are dependency-ready successors. Per project
planning convention, the registry registers only M066 as the next handoff;
M069 remains dependent on M066, M070 on M067/M068, and M072 on M066–M071.

Internal-only attestation: all repository writes are scoped to
`eggstack/emissary`. No upstream or third-party issue, review, merge,
submission, contribution package, or maintainer contact was prepared.

Final disposition: **closed**.
