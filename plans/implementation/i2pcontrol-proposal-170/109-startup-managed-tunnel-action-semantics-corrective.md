# M109 — Startup-Managed Tunnel Action Semantics Corrective

Status: **ready**

Class: corrective capability / lifecycle / containment

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Predecessor and security authority:

- M093 closure: `plans/closure/i2pcontrol-proposal-170/093-closure.md`
- M104 closure: `plans/closure/i2pcontrol-proposal-170/104-closure.md`
- M105 residual audit: `plans/implementation/i2pcontrol-proposal-170/105-residual-option-audit.toml`
- M108 closure: `plans/closure/i2pcontrol-proposal-170/108-closure.md`

Repository baseline:

- `2317705ef3bf21771715e243e87b62a6377a91eb` — post-M108 planning reconciliation.

Pinned external authority:

- I2P Proposal 170, `I2PControl Expansion`, status `Open`, revision `2026-05-20`.

All external sources are read-only evidence. This plan authorizes writes only to `eggstack/emissary` and does not authorize upstream interaction.

## 1. Objective

Correct the remaining canonical TunnelManager action mismatch for startup-configured generic tunnels without turning I2PControl into the owner of `router.toml` or broadening router-core lifecycle APIs.

The current production composition exposes startup-configured client/server tunnels through the I2PControl startup inventory, but marks them `StartupManaged` / `ExternallyManaged`. Canonical individual `start`, `stop`, and `restart` requests reject those names, and canonical `All=true` lifecycle operations skip them. This means the current action vocabulary exists but does not have literal all-visible-tunnel lifecycle semantics.

M109 must establish a bounded neutral lifecycle handle for the existing startup generic tunnel managers so that:

1. each startup tunnel has truthful running/stopped/starting/stopping observation;
2. canonical `start`, `stop`, and `restart` can act on a named startup tunnel;
3. canonical `All=true` includes startup and I2PControl-created tunnels in deterministic bounded order;
4. lifecycle state remains owned by the existing CLI tunnel layer and only the administrative mapping/policy stays in `i2pcontrol`;
5. no startup configuration file is rewritten by I2PControl.

M109 does **not** change the 70×12 option matrix. It must leave M095 at `224 apply / 158 blocked_primitive / 458 not_applicable`.

## 2. Why prior closure did not catch this as a separate gate

M095/M104 counted the seven canonical actions as implemented at the handler/control-plane level and separately tracked the option/type matrix. The startup inventory was intentionally introduced as read-only observation to avoid contaminating the existing startup managers. That preserved containment but allowed a narrower semantic mismatch: visible startup definitions are returned by TunnelManager while lifecycle dispatch rejects or skips them.

M104's local live fixture exercised control-plane-created tunnel lifecycle, not a mixed startup/control-plane `All=true` lifecycle set. M109 adds that missing mixed-inventory evidence.

## 3. Readiness and current evidence

M109 is dependency-ready because the required lower-layer building blocks already exist in the CLI tunnel layer:

- `emissary-cli/src/tunnel/client.rs::run_single_client` is a reusable cancellable client runtime primitive with readiness signaling;
- `emissary-cli/src/tunnel/server.rs::run_single_server` is a reusable cancellable server runtime primitive with readiness signaling and destination observation;
- startup composition currently spawns `ClientTunnelManager::run()` and `ServerTunnelManager::run()` from `emissary-cli/src/main.rs` but retains no lifecycle handle;
- `StartupTunnelInventory` already provides bounded name-indexed administrative definitions;
- `ProductionTunnelManagerControl` already serializes lifecycle by name and explicitly detects startup-managed names.

No `emissary-core`, `emissary-util`, Yosemite fork/vendor, Cargo dependency, protocol, frontend, or workflow change is required.

## 4. Invariants

M109 MUST preserve:

- exact Proposal 170 method/action spelling and response shape;
- exactly 12 canonical tunnel types and seven canonical actions;
- M095 counts `224 / 158 / 458`;
- Proposal 170 policy in `emissary-cli/src/i2pcontrol/**` wherever possible;
- startup tunnel runtime ownership in the existing CLI tunnel layer;
- no I2PControl writes to `router.toml` or startup destination-path configuration;
- no persistence of startup private destination material into TunnelStore/raw JSON;
- server destination/private-key secrecy and M093 loopback/anonymity boundaries;
- bounded per-name lifecycle serialization and cancellation;
- no lock held across SAM/network I/O, task joins, sleeps, or readiness waits;
- no double-running generation for one startup tunnel name;
- feature-disabled/default startup behavior byte-for-behavior compatible except for neutral reusable helper refactoring;
- startup tunnels continue to start automatically when I2PControl is disabled;
- no frontend ownership or UI control surface;
- internal-only repository interaction.

## 5. Explicit non-goals

M109 MUST NOT:

- implement any of the 158 residual option cells;
- change M095/M105 support dispositions;
- rewrite, migrate, or persist changes into `router.toml`;
- make I2PControl the canonical startup configuration parser;
- add a router-wide lifecycle manager;
- modify Yosemite or create a parallel SAM implementation;
- add or change Cargo dependencies or `Cargo.lock`;
- implement unrelated base I2PControl methods;
- alter AddressBook, RouterInfo, ClientServicesInfo, TLS, token, or frontend behavior;
- redesign the twelve I2PControl tunnel data planes;
- infer that startup `edit`/`delete` must mutate router configuration without direct pinned contract evidence.

## 6. Expected production paths

Preferred changes are limited to:

- `emissary-cli/src/tunnel/client.rs` — neutral startup client lifecycle handle/owner;
- `emissary-cli/src/tunnel/server.rs` — neutral startup server lifecycle handle/owner;
- `emissary-cli/src/main.rs` — retain and inject the neutral handle only when I2PControl is enabled;
- `emissary-cli/src/i2pcontrol/production.rs` — bounded startup lifecycle adapter and runtime-state projection;
- `emissary-cli/src/i2pcontrol/server.rs` — composition context wiring if required;
- `emissary-cli/src/i2pcontrol/tunnel_manager.rs` — canonical action dispatch semantics only;
- focused tests under the existing CLI/i2pcontrol suites.

No other production path is authorized. If implementation requires `emissary-core/**`, `emissary-util/**`, a dependency change, or a new router-global owner, stop and return to planning.

## 7. Work packages

### WP1 — Define a neutral startup lifecycle handle

Create the smallest CLI-tunnel-layer handle that can identify configured startup tunnels by exact name and perform lifecycle control without importing I2PControl domain types.

The handle must:

- use a bounded map whose maximum population is the existing startup tunnel count;
- own per-name cancellation/generation state;
- report a small neutral state enum or snapshot (`starting`, `running`, `stopping`, `stopped`, `failed` as needed);
- reject duplicate names deterministically;
- expose only start/stop/restart/status operations needed by the existing manager and I2PControl adapter;
- never expose private destination key material.

Do not add Proposal-170-named types to the CLI tunnel module.

### WP2 — Refactor startup managers onto the reusable cancellable primitives

Use `run_single_client` / `run_single_server` as the runtime execution units for lifecycle-controlled startup definitions where technically compatible.

Preserve current startup behavior:

- initial configured tunnels start automatically;
- existing retry/readiness behavior remains bounded;
- server destination observation still updates the startup inventory;
- shutdown/cancellation cannot leave a second live generation;
- a failed start releases its reservation and reports failure truthfully.

If the current shared client manager's one-session behavior cannot be preserved by a per-tunnel primitive without changing existing non-I2PControl semantics, keep the existing shared startup execution structure and add lifecycle control around it rather than silently changing identity/session sharing.

### WP3 — Wire lifecycle observation into `StartupTunnelInventory`

Replace fixed `ExternallyManaged` reporting with request-time state derived from the neutral handle.

The inventory remains configuration metadata plus public destination observation; runtime state must not be persisted as durable configuration.

### WP4 — Canonical named lifecycle

For startup-managed names, make canonical `start`, `stop`, and `restart` route through the startup lifecycle adapter instead of returning `externally managed`.

Requirements:

- same-name lifecycle remains serialized with the existing ProductionTunnelManagerControl lifecycle lock;
- start on already-running and stop on already-stopped must have deterministic Proposal-compatible textual status;
- restart must not overlap old/new generations;
- failures must not alter the durable control-plane TunnelStore.

### WP5 — Canonical `All=true`

Build one bounded deterministic target set from both inventories.

- deduplicate by exact tunnel name before dispatch;
- preserve the existing hard maximum (`MAX_TUNNEL_INVENTORY` or a stricter existing action bound);
- include startup-managed definitions rather than skipping them;
- execute through the same per-name lifecycle paths as individual requests;
- return per-target results using the existing Proposal 170 response contract;
- one target failure must not fabricate success for that target or corrupt other lifecycle owners.

Name collisions between startup and control-plane durable definitions must continue to fail or resolve according to the existing inventory ownership invariant; M109 must not invent shadowing between tunnel owners.

### WP6 — Reconcile startup `edit` / `delete` semantics

Perform a direct pinned Proposal/reference check limited to whether a startup-origin controller that is visible through canonical TunnelManager must also be mutable through `edit` and `delete`.

- If the pinned contract permits immutable externally configured definitions, retain explicit error semantics and document the ownership distinction.
- If the pinned contract requires successful mutation of every visible controller, **stop M109 closure** unless a durable implementation exists without rewriting `router.toml`, duplicating startup authority, or introducing an unreviewed overlay. Open a separately numbered architecture/capability corrective rather than hiding the requirement.

This work package may not reinterpret implementation difficulty as contract permission.

## 8. Failure, cancellation, restart, and contention semantics

Lifecycle state transitions must be generation-local. A stop/restart sends cancellation, waits with a finite bound for task termination, and only then permits a successor generation. If bounded cleanup fails, report failure and retain a state that prevents accidental duplicate start until the old generation is known gone.

No mutex may be held while awaiting SAM setup, listener binding, network traffic, cancellation completion, or backoff sleep. Use per-name operation serialization plus independently owned runtime state.

Application shutdown must cancel controlled startup runtimes without depending on the I2PControl HTTPS task still being alive.

Process restart reconstructs configured startup definitions from normal configuration and starts them according to existing startup semantics. Runtime stop state requested through I2PControl is not silently persisted into `router.toml`.

## 9. Compatibility and migration

No storage/schema migration is authorized.

With `i2pcontrol` disabled, startup tunnel creation and runtime behavior must remain equivalent to the pre-M109 baseline. With the feature compiled but runtime-disabled, no I2PControl lifecycle owner/state is constructed.

With I2PControl enabled, existing startup tunnel names remain visible but gain truthful lifecycle control and state. Control-plane-created tunnel persistence is unchanged.

## 10. Focused tests

Add focused evidence for at least:

- startup client start → running → stop → stopped → restart;
- startup server start → destination publication → stop → restart without secret leakage;
- named lifecycle requests for startup definitions no longer return `externally managed`;
- mixed startup + control-plane `All=true` start/stop/restart includes each eligible name exactly once;
- one failing target in `All=true` does not corrupt successful targets;
- duplicate concurrent start/restart of one startup name cannot create two live generations;
- cancellation during startup readiness releases state;
- startup runtime state is request-time truth, not a persisted fake;
- feature/runtime-disabled composition retains legacy startup behavior;
- no startup private destination appears in errors/debug/results.

Add a regression explicitly covering the pre-M109 defect: `All=true` must not silently skip a visible startup tunnel.

## 11. Broad verification

Run:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo check
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Record the known repository rustfmt/toolchain mismatch accurately; do not retain unrelated formatter churn.

No new hosted CI/fuzz/coverage loop is required.

## 12. Documentation and static guards

Update the active planning/docs only as needed to state that startup lifecycle action semantics are implemented or blocked by closure evidence.

M061/M062 containment must be amended only for the exact neutral CLI-tunnel and composition paths listed above. The closure must explicitly prove that no core/util/dependency/frontend path changed.

The M095 matrix remains unchanged because M109 corrects action/inventory semantics, not option/type support cells.

## 13. Acceptance criteria

M109 may enter closure only when:

1. individual canonical start/stop/restart works truthfully for startup-managed generic tunnels;
2. canonical `All=true` includes visible startup-managed and control-plane-owned targets exactly once;
3. startup runtime state is observed truthfully by TunnelManager;
4. lifecycle operations are bounded, serialized, cancellable, and duplicate-generation safe;
5. no startup configuration or private key material is rewritten by I2PControl;
6. default/runtime-disabled startup behavior is preserved;
7. M095 remains exactly `224 / 158 / 458`;
8. focused, feature, containment, matrix/audit, live-runtime, check and clippy evidence is green except explicitly recorded pre-existing tooling limitations;
9. WP6 has a direct pinned-contract disposition for startup edit/delete rather than an assumption;
10. all repository writes and closure evidence remain internal-only.

## 14. Stop conditions

Stop and open a new numbered corrective/architecture decision if:

- full startup mutation requires rewriting `router.toml` or a competing durable configuration overlay;
- client startup lifecycle control would silently change existing shared-session identity semantics;
- a required primitive belongs in `emissary-core` rather than the existing CLI tunnel owner;
- implementation requires vendoring/forking Yosemite or a new dependency;
- a lifecycle transition can leave two generations alive;
- a server secret would cross into the administrative inventory or logs;
- M093 anonymity/loopback/resource invariants would be weakened.

## 15. Closure evidence required

The M109 closure must contain:

- implementation commit(s) and exact changed paths;
- requirement-to-evidence table for named and `All=true` lifecycle;
- mixed-inventory live test evidence;
- edit/delete pinned-contract disposition;
- lifecycle failure/cancellation/restart review;
- security/secret/containment review;
- proof M095 counts are unchanged;
- exact verification outcomes;
- unresolved findings and severity;
- a dependency decision for the next registered handoff;
- internal-only external-source attestation.

## 16. Internal-only boundary

No upstream issue, pull request, review request, submission, adoption request, merge activity, branch/tag push, release, contribution preparation, or maintainer contact is authorized. External specifications and reference code are read-only evidence only.
