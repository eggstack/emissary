# M109 Closure — Startup-Managed Tunnel Action Semantics Corrective

Status: closed

Review date: 2026-09-01

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/109-startup-managed-tunnel-action-semantics-corrective.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Pinned Proposal 170 authority:

- I2P Proposal 170, `I2PControl Expansion`, status `Open`, revision `2026-05-20`;
- https://i2p.net/en/proposals/170-i2pcontrol-expansion/

## 1. Disposition and implementation head

M109 completed its bounded corrective objective in these implementation commits:

- `89d9b932462a590a1545e5c069932d97fe7e06e3` — `fix(i2pcontrol): control startup tunnel lifecycle actions`
- `cc2d0c94957475b497520f6db5a6adedffc787cc` — `fix(i2pcontrol): preserve shared startup client session`

Startup-configured generic client and server tunnels now have bounded neutral
controllers owned by the CLI tunnel layer. The I2PControl adapter projects
their request-time state and dispatches named and `All=true` lifecycle actions
without persisting startup runtime state or rewriting `router.toml`.

The controlled startup client path retains the existing manager's single
Yosemite streaming session and shares it across its lifecycle-owned tunnel
controllers; the default path remains unchanged. The full Proposal 170 status
remains partial because M095 still contains 158 applicable residual option
cells.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Bounded neutral lifecycle handle with exact-name ownership | `emissary-cli/src/tunnel/client.rs::StartupTunnelLifecycleHandle`; deterministic `BTreeMap`, duplicate rejection, 1000-entry bound | pass |
| Client start → running → stop → stopped → restart | `tunnel_client::lifecycle_tests::startup_client_lifecycle_is_generation_safe_and_restartable` with fake SAM; controlled composition injects the manager's single shared Yosemite session | pass |
| Server start, destination publication, stop, restart, and no secret result | `tunnel_server::tests::startup_server_lifecycle_publishes_and_restarts_without_exposing_secret`; observer receives only public destination | pass |
| Named canonical startup lifecycle dispatch | `ProductionTunnelManagerControl::{start,stop,restart}` routes startup-owned names through the neutral handle; lifecycle handler no longer rejects them | pass |
| Truthful request-time inventory state | `StartupTunnelInventory::list/get` map neutral state to internal runtime state; no runtime state is stored in `TunnelStore` | pass |
| Mixed startup/control-plane `All=true` target set | `handle_lifecycle_all` deduplicates exact names in deterministic order and dispatches every target; `tunnel_manager::tests::handler_all_lifecycle_includes_startup_targets_once` is the regression for the pre-M109 skip | pass |
| Failure/cancellation/restart generation isolation | Per-controller operation serialization, generation-local state, cancellation, bounded join/abort cleanup, and failed-cleanup start rejection in `client.rs`/`server.rs` | pass |
| Automatic startup and feature-disabled compatibility | Existing constructors retain the legacy manager path; lifecycle composition is injected only by the feature-enabled composition root; feature-enabled and no-feature checks pass | pass |
| Startup edit/delete ownership disposition | §5 below; handler and production control plane continue to reject mutation of startup-owned definitions | pass; explicit immutable ownership disposition |
| M095 unchanged | Matrix file SHA-256 remains `45d0e66d40e95f3caeb13f68050f93be8d9000b0666565399e1fb8eac6f2b6cb`; counts remain `224 / 158 / 458` | pass |

## 3. Exact changed paths and containment

The implementation commits changed exactly these paths in aggregate:

- `emissary-cli/src/tunnel/client.rs`
- `emissary-cli/src/tunnel/server.rs`
- `emissary-cli/src/main.rs`
- `emissary-cli/src/i2pcontrol/production.rs`
- `emissary-cli/src/i2pcontrol/tunnel_manager.rs`
- `emissary-cli/tests/m062_dependency_containment.rs`

The second implementation commit only refined the client lifecycle seam to
preserve the existing shared-session identity and did not add any path.

The non-I2PControl changes are the exact pre-authorized neutral CLI tunnel
and composition seams. No `emissary-core`, `emissary-util`, Yosemite source,
Cargo manifest, `Cargo.lock`, frontend, workflow, router configuration parser,
or tunnel data-plane family changed. M061/M062 containment passed after the
M109-specific exact-path guard was added.

## 4. Lifecycle and failure review

- Each registered name has one controller, one active generation, one
  cancellation sender, and at most one owned runtime task.
- Start, stop, and restart are serialized per name. Restart completes bounded
  cancellation and task termination before creating its successor.
- State transitions are generation-checked. A stale task cannot overwrite a
  newer generation's state.
- Readiness is reported only after the client listener/session or server
  session/forward path is ready. Setup failure reports failure and does not
  fabricate `running` state.
- Stop waits a finite bound and aborts a task that exceeds it. A failed cleanup
  remains `failed` and rejects a new start while the old task is still live.
- `All=true` continues after an individual failure and records that target's
  failure rather than fabricating success for the whole set.
- The existing per-name lifecycle guard and controller operation gate remain
  held for the bounded dispatch, preserving same-name serialization. Store and
  inventory locks are not held across controller readiness, SAM setup,
  cancellation, or task cleanup; the controller's state mutex is held only for
  bounded state/task ownership updates.

## 5. Startup edit/delete pinned-contract disposition

The pinned Proposal 170 TunnelManager section names `edit` and `delete` as
actions over I2PTunnel controllers and specifies their request/response
shapes. It does not define a startup-configuration ownership model, require
that preconfigured external definitions be mutable through I2PControl, or
authorize rewriting an implementation's startup configuration. The proposal's
general compatibility text also does not establish a durable overlay contract.

M109 therefore retains explicit immutable ownership semantics for startup
definitions: `edit` and `delete` return an error identifying startup
configuration ownership, while `get`, `start`, `stop`, `restart`, and mixed
`All=true` lifecycle operations observe or control the neutral runtime owner.
This is a truthful contract-silent ownership disposition, not a claim that
startup mutation is implemented. If a later pinned revision explicitly
requires mutation of externally configured definitions, a separately numbered
architecture/capability plan is required; M109 does not create a competing
durable overlay.

## 6. Security, secret, and compatibility review

- The lifecycle handle exposes only names, neutral state, and lifecycle
  operations. Server private destination material remains inside the server
  controller/runtime configuration and is not `Debug`, serialized, returned,
  or included in errors/results.
- Server destination observation continues to publish only the actual public
  destination to `StartupTunnelInventory`.
- No I2PControl action writes `router.toml`, startup destination files, or
  startup private destination material. Control-plane `TunnelStore` state is
  unchanged by startup lifecycle failures.
- The default/no-I2PControl constructor path retains the existing startup
  manager behavior. The lifecycle owner is created and injected only by the
  feature-enabled composition path.
- M093 loopback/anonymity, destination, path, and secret boundaries remain
  unchanged. No core/util/dependency/frontend path changed.

## 7. Verification outcomes

Passed:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
  657 passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
  1793 passed across 26 suites
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m023_startup_inventory --test m033_tunnel_lifecycle --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
  36 passed across 6 suites
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
  1 passed
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
  pass
cargo check
  pass
git diff --check
  pass
```

The required formatter command was run exactly:

```text
cargo fmt --all -- --check
  fails with exit code 1
```

The failure is the established repository stable/nightly rustfmt
configuration mismatch and pre-existing formatting differences outside M109,
including nightly-only option warnings. Formatter-only churn was removed and
was not included in the implementation commit.

## 8. Future-plan dependency audit

No future plan becomes ready from M109 closure.

- M110 remains `proposed / blocked`: M109 is now closed, but M110 still
  requires explicit acceptance of its bounded I2PControl-local shared-session
  and destination/key ownership model plus proof that accepted Yosemite APIs
  can consume the required material.
- M111 remains `proposed / dependency-blocked` on an accepted public Yosemite
  session-wire option path.
- M112 and M113 remain blocked on their separately named client lifecycle/
  proxy and server presentation/LeaseSet primitives.
- M114 remains blocked until M109–M113 are closed as applicable and M095/M105
  have zero unresolved applicable residuals.

The registry and roadmap now record M109 as closed, leave no successor
registered or ready, and preserve M110–M114 as roadmap-only blocked plans.

## 9. Unresolved findings and final disposition

No M109-scoped implementation finding remains. The Proposal 170 option matrix
and full-support claim remain intentionally partial as recorded by M095/M105.
The immutable startup `edit`/`delete` disposition is explicit and requires a
new architecture/capability plan only if a later pinned contract changes it.

M109 is formally **closed** against implementation head
`cc2d0c94957475b497520f6db5a6adedffc787cc`, with
`89d9b932462a590a1545e5c069932d97fe7e06e3` as its initial implementation
commit.

## 10. Internal-only attestation

External Proposal 170 material was accessed read-only for contract evidence.
All repository writes remained within the authorized internal
`eggstack/emissary` repository. The user explicitly authorized committing and
pushing this work to that configured internal fork. No upstream repository,
maintainer channel, issue, pull request, review, merge, adoption request,
submission, release, or contribution artifact was created or mutated.
