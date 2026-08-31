# M106 Closure — DelayOpen Client-Listener Lifecycle

Status: **closed**

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/106-delay-open-client-listener.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Pinned Proposal 170 revision: `2026-05-20` (status `Open`).

Review date: 2026-08-31.

## 1. Disposition and exact implementation head

M106 completed its bounded I2PControl-local implementation objective. The
implementation commit is:

- `4f5bd42cdd6991dc5c225d42b9ddf0490aa32559` — `feat(i2pcontrol): implement delayed client listener sessions`

The implementation preserves eager startup when `DelayOpen` is omitted or
false. When true for one of the six supported TCP-style client families, the
existing generation-local listener binds and reports its local address before
the first Yosemite session is created. A generation-local `OnceCell` serializes
concurrent first use and shares only that generation's session with bounded
handlers. Setup failure signals the listener, fails the triggering connection,
and enters the existing failed runtime state; cancellation wins
deterministically and drains bounded handlers.

The final production matrix was updated from the M105/M104 baseline only for:

| Option | `client` | `httpclient` | `ircclient` | `socks` | `socksirc` | `connectclient` | `streamrclient` |
|---|---|---|---|---|---|---|---|
| `DelayOpen` | apply | apply | apply | apply | apply | apply | blocked — no pinned first-local-client-socket event |

The current 840-cell TunnelManager inventory is therefore:

- 224 `apply`;
- 158 applicable `blocked_primitive`;
- 458 `not_applicable`;
- 0 `planned_apply`, unsupported, unknown, or accept-inert cells.

Authoritative matrix SHA-256 at closure: `45d0e66d40e95f3caeb13f68050f93be8d9000b0666565399e1fb8eac6f2b6cb`.

M104 is not reopened or claimed successful. Its historical final review was
218 `apply` / 164 `blocked_primitive` / 458 `not_applicable`; M106 resolves
only the six explicitly promoted cells.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Typed `DelayOpen` option | `TunnelOptions::delay_open`, canonical parser, merge, typed get serialization | pass |
| Lossless create/edit/get persistence | `tunnel_manager` extraction/merge/output paths and canonical wire fixture | pass |
| Six-family capability advertisement | client, HTTP client, IRC client, SOCKS/SOCKS-IRC, and CONNECT capability declarations; Streamr declaration remains without `DelayOpen` | pass |
| Fail-before-allocation validation | capability validation rejects unsupported `DelayOpen` before backend construction; Streamr regression test | pass |
| Bind before delayed allocation | shared `run_client_listener` delayed branch and idle SAM-count test | pass |
| No idle `HELLO`/`SESSION CREATE` | fake-SAM `delayed_listener_binds_without_sam_until_first_connection` | pass |
| Exactly one concurrent first session | `concurrent_first_connections_create_one_session`; one `SESSION CREATE` observed | pass |
| Cancellation before first connection | `cancellation_before_first_connection_does_not_create_session` | pass |
| Cancellation during setup | gated fake-SAM `cancellation_during_lazy_setup_returns_deterministic_setup_error` | pass |
| Failed setup recovery/state | `failed_lazy_setup_fails_the_generation`; setup error reaches listener and generation exits failed | pass |
| Restart/edit generation isolation | `restarted_generation_does_not_reuse_the_prior_session`; fresh session per listener generation | pass |
| Bounded handlers and shutdown | existing `BoundedTaskGroup`, listener drain path, full feature suite | pass |
| Existing eager behavior | pre-existing listener lifecycle tests and all non-delayed configurations | pass |
| Streamr exclusion | Streamr capability remains rejecting; no Streamr source/runtime files changed | pass |

## 3. Changed-path and containment review

Implementation and guard changes were confined to the existing I2PControl
runtime/composition boundary plus the authoritative matrix and its planning
guards:

- `emissary-cli/src/i2pcontrol/domain/tunnel.rs`
- `emissary-cli/src/i2pcontrol/tunnel_manager.rs`
- `emissary-cli/src/i2pcontrol/backends/options.rs`
- `emissary-cli/src/i2pcontrol/backends/runtime/client_listener.rs`
- the six approved TCP-client composition seams, with the existing
  `http_bidir` caller explicitly retaining eager behavior for API completeness
- `emissary-cli/tests/m095_full_support_matrix.rs`
- `emissary-cli/tests/m105_residual_option_audit.rs`
- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`

No Cargo manifest, lockfile, Yosemite source, core/util crate, dependency,
frontend, workflow, Streamr runtime, wire format, destination/key format,
TLS trust rule, proxy fallback, DNS behavior, or server target boundary
changed. M061/M062 containment remains satisfied.

## 4. Verification outcomes

The following passed against implementation head `4f5bd42c`:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib client_listener::tests --no-fail-fast
  pass: 9 tests

cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib delay_open --no-fail-fast
  pass: 2 tests

cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
  pass: 2 suites

cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
  pass: 1773 tests across 26 suites

cargo check
  pass

cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
  pass: no issues found

cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --no-fail-fast
  pass: 26 tests across 2 suites

git diff --check
  pass
```

`cargo fmt --all` was run. The repository's documented stable/nightly
rustfmt mismatch emitted the known unstable-option warnings and would produce
formatter-only churn outside this handoff on stable; that unrelated churn was
removed. No formatter-only core changes were retained.

The required containment and feature-gated checks passed: the two containment
suites passed 26 tests, and the feature-gated clippy check reported no issues.

## 5. Compatibility, security, and lifecycle review

Definitions without `DelayOpen` retain eager session allocation and existing
startup errors. Delayed mode changes only the allocation point. It does not
change SAM wire fields, destination identity, target validation, direct-I2P
DNS behavior, proxy selection, TLS trust, or server routing.

The owner is bounded by listener generation and `max_connections`; no mutex is
held over network I/O, cancellation, or task joins. A failed lazy setup is
stored for that generation only and cannot leak into a later restart. A
cancelled setup returns `SessionSetup` to the triggering handler while the
listener's cancellation branch wins over the setup-failure notification, so
stop/edit/restart remains a normal stopped transition. No session is allocated
while a delayed listener is idle.

Streamr remains intentionally separate: its UDP/session loop has no canonical
first local TCP client socket, its option is still blocked, and its documented
subscriber, expiry, payload, transport-buffer, refresh, and shutdown limits
are unchanged.

## 6. Future-plan disposition

M106 unblocks no future implementation plan. M104 remains **closed as
blocked** because 158 applicable `blocked_primitive` cells remain in the
updated M095 matrix; its required zero-residual gate and live/reseeded/reference
router reclosure have not been met. No new successor is registered. The
remaining residual families stay deferred or blocked pending new exact
dependency, semantic, or architecture evidence.

The planning controls now record:

- M106: closed, with this closure;
- current production inventory: 224 apply / 158 blocked / 458 not-applicable;
- Proposal 170 support: still partial;
- next dependency-ready handoff: none.

## 7. Internal-only attestation and final disposition

All repository writes remained internal to `eggstack/emissary`. Proposal 170,
Yosemite, and prior reference material were used only as read-only evidence.
No upstream repository, issue, pull request, review, adoption request,
maintainer channel, release, or external service was mutated. No upstream
contribution artifact was prepared.

M106 is therefore **closed**. The six TCP-client `DelayOpen` cells are
operational through the existing I2PControl client-listener owner, Streamr is
explicitly excluded, and full Proposal 170 support remains correctly partial.
