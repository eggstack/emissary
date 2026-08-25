# M087 Closure — Generic Server Inactivity Timeout Corrective

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/087-generic-server-inactivity-timeout-corrective.md`

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

M087 baseline: `7dc8d719e48e31637b8640645510d6f71b0939d8`.

Final reviewed M087 implementation head:
`60e929acb482aba4cec46b88522b79ce247a0156`.

The implementation commit changes only the generic server backend. This closure
record and the associated planning-status updates are the subsequent internal
planning bookkeeping for that reviewed implementation head.

## 1. Disposition

M087 is complete. Generic Proposal 170 `server` accepted streams no longer
retain a shared admission lease indefinitely when neither relay direction makes
byte-transfer progress. The raw stream remains active for as long as progress
continues, without a maximum total connection age.

No new Proposal 170 field, option spelling, dependency, core/router/startup,
frontend, or external repository interaction was introduced.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Zero-progress generic relays have a finite bound | `relay_with_inactivity` uses a resettable Tokio deadline; `generic_server_idle_expiry_releases_admission_lease` advances paused time through the full interval | pass |
| The bound is inactivity/progress-based, not absolute age | Deadline resets only after `read` returns bytes and `write_all` succeeds; `generic_server_progress_resets_deadline_without_fixed_lifetime` stays alive across multiple intervals | pass |
| Progress in either direction resets the deadline | The two-direction progress test and `generic_server_unidirectional_progress_resets_deadline` exercise remote-to-target and target-to-remote progress | pass |
| Readiness/wakeup without bytes is not progress | `generic_server_readiness_wakeup_without_progress_does_not_extend_deadline` supplies a spurious wakeup with no bytes and observes expiry | pass |
| Half-close remains useful and bounded | Each direction shuts down its opposite writer on EOF, while the other direction continues; `generic_server_half_close_drains_the_other_direction` verifies request EOF followed by response drain and final completion | pass |
| Five-second target connect remains intact | `bounded_target_connect` retains `TARGET_CONNECT_TIMEOUT = 5s`; `generic_server_target_connect_timeout_remains_five_seconds` verifies paused-time expiry | pass |
| The existing admission lease remains authoritative | The handler task still owns the lease around the relay; the idle-expiry test reacquires the sole admission slot only after relay completion | pass |
| Timeout/error/EOF paths unwind tasks and resources | Relay futures are task-local and dropped on timeout/error; timeout, EOF, and half-close tests await completion, and the accepted handler continues to ignore only the relay result | pass |
| Diagnostics do not disclose peer/private Destination material | Relay code creates no peer-bearing error text; `generic_server_peer_diagnostics_are_redacted` verifies peer Debug redaction and the existing `generic_server_debug_is_secret_safe` covers private server material | pass |
| Scope and API containment | Implementation diff is only `emissary-cli/src/i2pcontrol/backends/server.rs`; no wire/API/dependency/core/router/startup/frontend change | pass |

## 3. Timeout selection and relay semantics

The chosen internal interval is exactly **10 minutes**:

```text
GENERIC_SERVER_INACTIVITY = Duration::from_secs(10 * 60)
```

This follows the existing IRC server's ten-minute post-registration
activity-resetting bound, while remaining materially longer than the existing
five-second local-target connect deadline and the HTTP server's short request
and header deadlines. It is long enough for ordinary interactive and
long-lived raw tunnel protocols, but it prevents a peer that makes no progress
from pinning an admission slot forever. The interval is internal only; no
Proposal 170 contract field or unrelated option configures it.

The old behavior connected to the loopback target with the existing five-second
timeout and then delegated the entire lifetime to unbounded
`copy_bidirectional`. The new behavior retains the same target confinement and
connect deadline, then runs two task-local relay directions under one shared
inactivity deadline. A successful read followed by successful forwarding
resets that deadline. EOF shuts down the opposite write half and only disables
that completed direction; the other direction may drain normally. If both
directions finish, an I/O error occurs, or inactivity expires, the handler
returns and its existing admission lease drops. No absolute connection-age
limit was added.

## 4. Changed paths and containment

Implementation commit:

- `emissary-cli/src/i2pcontrol/backends/server.rs`

Closure/planning bookkeeping:

- `plans/closure/i2pcontrol-proposal-170/087-closure.md`;
- `plans/implementation/i2pcontrol-proposal-170/087-generic-server-inactivity-timeout-corrective.md`;
- `plans/implementation/i2pcontrol-proposal-170/088-pre-accept-server-admission-boundary-corrective.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/registry.md`;
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`.

No `emissary-core/**`, router, startup, frontend, manifest, dependency,
lockfile, or Proposal 170 wire/API path changed. M062's existing exact-path
planning allowlist already covers the M087/M088/M089 implementation and
closure documents; its production path rules were not broadened.

## 5. Verification

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol generic_server
  -> pass (24 tests; 1664 filtered, 23 suites)

cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
  -> pass (19 tests)

rustfmt --check emissary-cli/src/i2pcontrol/backends/server.rs
  -> pass (stable-toolchain warnings only for unsupported repository-wide unstable rustfmt settings)

git diff --check
  -> pass
```

The focused test command covers the colocated generic-server tests and the
existing accepted-stream raw-relay regression. The M062 test confirms exact
dependency/source containment and unchanged lockfile policy.

## 6. Findings and successor disposition

No unresolved high- or medium-severity finding remains within M087's approved
scope. The lower-layer/pre-accept admission question is not silently treated as
resolved by this post-accept lifetime bound; it remains the explicit subject of
M088.

Planning status after closure:

- M087: **closed**;
- M088: **ready** — its administrative M087 dependency is satisfied and its
  separate Yosemite/Emissary/SAM capability investigation can proceed;
- M089: **future/blocked** — it remains verification-only and requires accepted
  M088 closure.

No other future plan was unblocked by M087 evidence. The accepted source/
truthfulness limitations, RouterInfo 37/1/5 disposition, M051 blocker, and
unrelated AddressBook/base-I2PControl gaps remain unchanged.

## 7. Internal-only attestation

All implementation, testing, closure, and planning writes were confined to the
internal `eggstack/emissary` repository. No upstream issue, pull request,
review, submission, merge request, maintainer contact, or contribution artifact
was opened, drafted, requested, or pushed. Any external specifications or
reference sources remain read-only evidence.
