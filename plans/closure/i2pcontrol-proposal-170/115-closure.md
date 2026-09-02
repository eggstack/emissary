# I2PControl Proposal 170 Milestone M115 — Closure Status

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/115-m109-runtime-disable-and-lifecycle-truthfulness-corrective-pass.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Repository baseline reviewed: `ee3b444` (implementation head)

Implementation commits:

- `ee3b444` — correct runtime-gated startup lifecycle composition, truthful lifecycle snapshots, retryable shared startup-client sessions, and focused regressions.

## 1. Executive finding

M115 is closed. The M109 post-closure defects F1–F5 are corrected within the
authorized seam. Runtime-disabled feature-capable startup now uses the
historical client/server managers; enabled startup retains named lifecycle and
mixed `All=true`; state reads use last-committed atomic snapshots; and the
controlled startup-client session has retryable creation and explicit
active-member lifetime. Planning records now identify M110 as the sole ready
successor while preserving the partial Proposal 170 status and the unchanged
`224 / 158 / 458` matrix.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| F1/F2 runtime-disable containment | `main.rs` runtime branch; `runtime_disabled_does_not_select_controlled_startup_path` | pass | Inventory, lifecycle handle, observer, and controlled constructors are created only in the enabled branch. |
| F3 truthful contention state | client/server lifecycle snapshot tests | pass | A contended async state mutex still returns committed `Stopped`, never synthetic `Starting`. |
| F4 retryable/lifecycle-complete shared session | `shared_startup_session_recovers_and_releases_by_membership`; fake-SAM attempt counter | pass | First failure retries; two members use one session; partial stop retains it; final stop releases it; restart creates a successor. |
| F5 planning state | registry, roadmap, implementation README, M115 plan, and this closure | pass | Stale M115-ready text is closed and M110 is the only ready handoff. |
| Enabled named `start`/`stop`/`restart` | M033 lifecycle suite and full feature-enabled package suite | pass | Existing M109 behavior remains operational. |
| Enabled mixed `All=true` | full feature-enabled package suite, including M033/tunnel-manager regressions | pass | Startup and control-plane names remain bounded and deterministic. |
| Feature absent behavior | `cargo check -p emissary-cli --no-default-features` | pass | Historical no-feature composition compiles. |
| Feature compiled + runtime disabled | static composition guard plus branch inspection | pass | The disabled branch selects `ClientTunnelManager::new` and `ServerTunnelManager::new`; no M109 owner is constructed. |
| Feature compiled + runtime enabled | feature-enabled checks, 660 library tests, 1,800 package tests, live fixture | pass | Enabled composition and lifecycle behavior remain operational. |
| M095/M105 unchanged | matrix/audit tests and planning review | pass | Counts remain exactly `224 / 158 / 458`; M115 owns zero residual cells. |
| Secret/anonymity boundaries | existing M061/M062/M093 authority, server tests, source review | pass | No private server destination crosses the neutral observer; no local-target/proxy/LeaseSet boundary changed. |
| Containment | M062 dependency-containment suite and changed-path review | pass | Only the exact M115 seam and planning/test evidence changed. |

## 3. Production implementation evidence

`emissary-cli/src/main.rs` computes `i2pcontrol_enabled` immediately after
configuration extraction and gates all M109-only startup inventory, lifecycle,
destination-observer, and manager construction on that runtime value. The
feature-disabled path remains the historical constructor path.

`emissary-cli/src/tunnel/client.rs` replaces `OnceCell` pre-seeding with a
bounded owner containing member reservations, one in-flight creator, explicit
release, and retry-after-failure behavior. Yosemite I/O occurs outside owner
bookkeeping locks. The owner is startup-only and is not exposed as Proposal
`Shared` behavior.

Client and server controllers mirror every committed lifecycle transition into
an atomic snapshot. Synchronous state observation is therefore non-blocking
and truthful during internal mutex contention. Generation checks remain in the
async task completion path.

`emissary-cli/src/tunnel/server.rs` changes only neutral state observation;
destination generation, public-destination observation, private-key handling,
and cancellation behavior remain otherwise unchanged.

## 4. Verification executed

### Commands run

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m023_startup_inventory --test m033_tunnel_lifecycle --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test static_guards --no-fail-fast
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

### Results

- Feature-enabled check: pass.
- No-feature check: pass.
- Workspace/default check: pass.
- Library tests: 660 passed.
- Full feature-enabled `emissary-cli` package suite: 1,800 passed across 26 suites.
- Required focused M023/M033/M061/M062/M095/M105 groups: 36 passed across 6 suites.
- Static guards: 43 passed.
- Live runtime fixture: 1 passed.
- Clippy with `-D warnings`: pass; no issues.
- `git diff --check`: pass.
- `cargo fmt --all -- --check`: fails on the established stable/nightly
  rustfmt configuration mismatch and pre-existing formatting outside this
  change. The command reports nightly-only options such as
  `imports_granularity`, `trailing_comma`, and `wrap_comments`; no unrelated
  formatter churn is retained.

## 5. Invariant review

- Proposal methods, actions, tunnel types, response shapes, and option policy are unchanged.
- M095 remains `224 apply / 158 blocked_primitive / 458 not_applicable`.
- M109 named lifecycle and mixed `All=true` remain enabled-runtime behavior.
- Runtime-disabled feature-capable startup follows the historical path.
- One active generation per startup name and generation-safe stale completion remain enforced.
- Shared-session transitions are serialized without holding bookkeeping locks across network I/O, sleeps, joins, or relay lifetime.
- Final-member release is explicit; failed creation does not poison later starts.
- Server private destination material remains controller-local and only the public destination is observed.
- No frontend ownership, persistence migration, router-global session owner, or Proposal-shaped lower-layer API was added.

## 6. Failure and recovery review

Creation failure removes the failed member reservation and returns a redacted
setup error. A later start can elect a new creator. Concurrent starts wait on a
notification and reuse the resulting session. A partial stop removes only its
member; the final stop drops the owner-held session after the generation task
has completed. Same-name operations remain serialized, restart waits for old
generation cancellation, and stale task completion is generation-checked.

Task panic/timeout stop paths also release the member reservation. Session
bookkeeping never spans Yosemite session creation or stream relay I/O. The
fake-SAM regression covers one failed attempt, recovery, concurrent active
members, partial release, final release, and successor creation.

## 7. Migration and compatibility review

No schema, configuration, startup destination-file, Cargo, lockfile, Yosemite,
core, or util migration occurred. No-feature and feature-capable runtime-
disabled startup retain historical manager construction. Enabled startup keeps
M109 lifecycle semantics. A later restart after final release may create a new
transient Yosemite session, which is intentional and is not M110 `NewDest` or
`PersistentClientKey` support.

## 8. Security review

Runtime compilation alone no longer instantiates M109 lifecycle/session owners
or creates long-lived network activity. Session errors are fixed redacted text;
private destination material is not logged or returned. Server destination
secrecy and public-only observation remain intact. M093 local-target, proxy,
HTTP/IRC, Streamr, admission, anonymity, and trusted-peer boundaries were not
changed. No M110–M113 option behavior was introduced.

## 9. Documentation and operations

Updated planning control surfaces:

- `plans/registry.md` — M115 closed; M110 sole ready/registered successor.
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` — M115 closed and M110 readiness recorded.
- `plans/implementation/i2pcontrol-proposal-170/README.md` — current state reconciled.
- `plans/implementation/i2pcontrol-proposal-170/115-m109-runtime-disable-and-lifecycle-truthfulness-corrective-pass.md` — closed status.
- `emissary-cli/tests/m062_dependency_containment.rs` — exact M115 path bookkeeping.

The stale pre-M109 statements remain only in the historical M109 plan as
baseline evidence; active registry, roadmap, README, and M115 closure text
describe the corrected lifecycle behavior.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | Stable rustfmt cannot satisfy the repository’s nightly-only configuration | Formatter check remains environmental/toolchain-limited | Run the documented nightly formatter in an environment that supports the configured options; do not retain unrelated churn. |

No M115-scoped high- or medium-severity finding remains.

## 11. Roadmap disposition

M115 is **closed**. M110 is now eligible and registered as the sole ready
successor after an independent readiness audit: the bounded I2PControl-local
ownership model is explicitly accepted, exact M095/M105 residual ownership is
frozen, and accepted Yosemite 0.7.0 publicly exposes `DestinationKind::Persistent`
and `SessionOptions` primitives sufficient for M110’s destination-material
handoff without dependency changes.

M111 remains dependency-blocked on accepted public session-wire capability.
M112 and M113 remain blocked on their independent residual owners and M110/M111
gates. M114 remains blocked until the residual option cells are resolved and
the final live/reference reclosure prerequisites are met. M115 does not change
the Proposal 170 full-support claim or residual matrix.

## 12. Registry updates

The registry now records M115 as closed and M110 as the sole ready/registered
handoff. M111–M114 remain roadmap-only blocked plans. The roadmap and
implementation README match those statuses, and no stale active statement
claims that M109 lifecycle actions reject or skip visible startup tunnels.

## 13. Internal-only attestation

External Proposal/reference/dependency materials were accessed read-only.
All writes stayed within `eggstack/emissary`. No upstream repository, issue,
pull request, review, maintainer channel, release, branch/tag, contribution
package, merge/adoption request, or submission artifact was created or
mutated.
