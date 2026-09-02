# M115 — M109 Runtime-Disable and Lifecycle-Truthfulness Corrective Pass

Status: **ready**

Class: corrective capability / lifecycle / containment / reliability

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Corrective authority and predecessor evidence:

- M109 plan: `plans/implementation/i2pcontrol-proposal-170/109-startup-managed-tunnel-action-semantics-corrective.md`
- M109 closure: `plans/closure/i2pcontrol-proposal-170/109-closure.md`
- M093 security closure: `plans/closure/i2pcontrol-proposal-170/093-closure.md`
- M061/M062/M063 containment authority
- M095 full-support matrix and M105 residual-option audit

Repository baseline:

- `fa25f194a919d52c76f298c640688697a15f66b3` — `docs(i2pcontrol): close M109 startup lifecycle corrective`

Pinned external authority:

- I2P Proposal 170, `I2PControl Expansion`, status `Open`, revision `2026-05-20`.

All external sources are read-only evidence. This plan authorizes writes only to `eggstack/emissary` and does not authorize upstream interaction.

## 1. Objective

Correct the bounded M109 lifecycle/composition defects discovered after closure without reopening unrelated Proposal 170 scope or expanding the lower-layer ownership boundary.

M109 successfully added neutral lifecycle control for startup-configured generic tunnels, but post-closure review found four defects in the implementation boundary:

1. a binary compiled with the `i2pcontrol` feature selects the M109 lifecycle-controlled startup managers even when runtime configuration has I2PControl disabled;
2. startup lifecycle state observation can fabricate `Starting` when the controller state mutex is merely contended;
3. the controlled startup-client shared Yosemite session is seeded once by the manager and cannot recover correctly from initial session-creation failure, while its teardown/lifetime after the final controlled client stops is not owned explicitly;
4. planning control text still contains a stale pre-M109 statement that startup lifecycle actions reject/skip startup tunnels.

M115 must restore the intended M109 boundary:

- **feature compiled + runtime disabled** behaves through the historical startup tunnel path, not the M109 controlled path;
- **runtime enabled** retains M109 named lifecycle and `All=true` behavior;
- lifecycle state returned to I2PControl is truthful even during contention;
- the single shared startup-client Yosemite session has explicit, retryable, bounded ownership tied to active controlled startup clients;
- planning state matches production reality.

M115 does **not** change the Proposal 170 option matrix. M095 must remain exactly:

- `224 apply`;
- `158 blocked_primitive`;
- `458 not_applicable`.

## 2. Findings requiring correction

### F1 — runtime-disable containment is keyed to compilation, not runtime enablement

`emissary-cli/src/main.rs` computes `i2pcontrol_enabled`, but startup inventory/lifecycle construction and the selection of `ClientTunnelManager::new_with_lifecycle` / `ServerTunnelManager::new_with_lifecycle` are guarded by `#[cfg(feature = "i2pcontrol")]` rather than the runtime boolean.

Consequences:

- a feature-capable binary with `i2pcontrol.enabled = false` still uses the M109 lifecycle execution path;
- M109-only startup inventory/lifecycle validation can affect a runtime where I2PControl is disabled;
- the security-audited historical startup path is not preserved under the runtime-disabled state promised by M109.

Disposition: **medium correctness/containment defect**.

### F2 — startup inventory/lifecycle is constructed before the runtime enablement decision is applied

The M109 composition currently creates `StartupTunnelInventory` and `StartupTunnelLifecycleHandle` whenever the feature is compiled. The runtime enabled flag is evaluated later.

Even if no I2PControl server is started, disabled runtime configuration may therefore be subjected to I2PControl-only inventory bounds/name ownership rules and alternate tunnel-manager construction.

Disposition: part of F1; must be fixed by one runtime-gated composition boundary rather than piecemeal special cases.

### F3 — lifecycle state observation fabricates `Starting` under mutex contention

Both startup lifecycle controller implementations expose synchronous state through a `try_lock()` fallback. If the mutex is contended, the fallback is `StartupTunnelState::Starting` regardless of the actual state.

This violates the Proposal 170 workstream invariant that unavailable/contended observation must not be represented as fabricated runtime state.

Disposition: **low/medium truthfulness defect**.

### F4 — controlled shared-client session ownership is not recoverable or lifecycle-complete

The M109 follow-up correctly restored one Yosemite streaming session shared by controlled startup clients, but the current owner is an `OnceCell` populated once by `ClientTunnelManager::run()`.

Two problems follow:

1. if initial shared-session creation fails, `start_all()` fails because the shared session is absent, and later individual `start` requests have no owner capable of retrying session creation;
2. after all controlled startup-client listeners are stopped, the shared session remains retained through controller-held references for the lifetime of the startup lifecycle handle. Its teardown semantics are therefore implicit rather than tied to active controlled clients.

M115 must preserve the legacy one-session sharing property while giving that session an explicit retryable lifecycle owner.

Disposition: **medium lifecycle/recovery defect**.

### F5 — planning-state drift

`plans/registry.md` still states that visible startup tunnels reject named lifecycle operations and are skipped by `All=true`, despite M109 closure recording the opposite.

Disposition: documentation/control-surface defect; correct in this plan-registration commit.

## 3. Why M109 verification missed these defects

M109 verification covered feature-enabled and feature-disabled compilation, direct lifecycle controllers, mixed `All=true`, and the live I2PControl fixture. It did not cover the distinct runtime state:

`i2pcontrol feature compiled + i2pcontrol.enabled = false`.

That allowed compile-time feature selection to masquerade as runtime containment.

The client lifecycle test constructs a controller directly and therefore exercises the independent-session path. It does not exercise the full `ClientTunnelManager::new_with_lifecycle` shared-session owner across:

- initial shared-session failure;
- later recovery by a named `start`;
- two simultaneously active startup clients;
- stopping one member while another remains active;
- stopping the final member;
- restarting from zero active members.

State tests also did not force observation while the controller state lock was contended, so the `try_lock() -> Starting` fallback was not detected.

M115 regression evidence must exercise those missing states directly.

## 4. Readiness and architecture decision

M115 is dependency-ready.

The required owners already exist in the exact M109 seam:

- `emissary-cli/src/main.rs` owns composition and already computes runtime `i2pcontrol_enabled`;
- `emissary-cli/src/tunnel/client.rs` owns the startup client manager, neutral lifecycle controllers, and shared Yosemite client session;
- `emissary-cli/src/tunnel/server.rs` owns startup server lifecycle state;
- `emissary-cli/src/i2pcontrol/production.rs` adapts neutral startup state/actions into Proposal 170 administrative state;
- `emissary-cli/src/i2pcontrol/tunnel_manager.rs` already contains the M109 action regressions and should require no semantic expansion.

The corrective architecture is intentionally narrow:

1. runtime enablement decides whether the M109 lifecycle owner is constructed/selected;
2. disabled runtime uses the historical constructors and execution path;
3. enabled runtime uses the neutral lifecycle path;
4. state observation uses a truthful non-fabricating snapshot owned by the lifecycle controller;
5. controlled startup clients share one retryable session owner while one or more members are active, with deterministic release when the final active member stops.

No router-core API, Yosemite fork/change, Cargo dependency, new persistence owner, or Proposal-shaped lower-layer API is required.

If implementation requires any of those, stop and return to planning.

## 5. Invariants

M115 MUST preserve:

- exact Proposal 170 method/action spelling and response shape;
- exactly 12 canonical tunnel types and seven canonical actions;
- M095/M105 dispositions and counts `224 / 158 / 458`;
- M109 named startup `start`/`stop`/`restart` and mixed `All=true` semantics when runtime I2PControl is enabled;
- M109 immutable startup `edit`/`delete` disposition;
- no I2PControl writes to `router.toml` or startup destination files;
- no persistence of startup private destination material in TunnelStore/raw JSON;
- server destination/private-key secrecy;
- M093 local-target, anonymity, HTTP/IRC/Streamr, and admission boundaries;
- one active generation per startup tunnel name;
- bounded cancellation/readiness and no stale generation state overwrite;
- no fabricated runtime state;
- historical startup tunnel behavior when runtime I2PControl is disabled, even if the binary contains the feature;
- historical no-feature behavior;
- one shared Yosemite client session for concurrently active controlled startup client tunnels, preserving the pre-M109 startup manager sharing model;
- no lock held across unrelated network I/O, sleeps, joins, cancellation waits, or filesystem synchronization;
- no frontend ownership;
- internal-only repository interaction.

## 6. Explicit non-goals

M115 MUST NOT:

- implement or reclassify any of the 158 residual option cells;
- implement Proposal 170 `Shared`, `NewDest`, `PersistentClientKey`, or `PrivKeyFile` semantics owned by M110;
- change Yosemite session option serialization owned by M111;
- implement client proxy/profile/timer residuals owned by M112;
- implement server/LeaseSet residuals owned by M113;
- change startup `edit`/`delete` ownership policy;
- rewrite or migrate `router.toml`;
- introduce a router-global lifecycle/session/key owner;
- change `emissary-core/**` or `emissary-util/**`;
- vendor/fork/patch Yosemite or create a parallel SAM implementation;
- change `Cargo.toml`, `Cargo.lock`, workflows, release automation, frontend/UI, AddressBook, RouterInfo, ClientServicesInfo, authentication, or TLS behavior;
- add a general session pool for control-plane-created Proposal 170 tunnels;
- broaden local listener/target interfaces or network reachability;
- fix unrelated rustfmt/tooling churn;
- initiate or prepare upstream review, contribution, merge, submission, release, or maintainer contact.

## 7. Required production paths

Authorized production changes are limited to the existing M109 seam:

- `emissary-cli/src/main.rs` — runtime-enabled composition selection only;
- `emissary-cli/src/tunnel/client.rs` — neutral client lifecycle state and shared-session owner/recovery;
- `emissary-cli/src/tunnel/server.rs` — neutral server lifecycle state observation only;
- `emissary-cli/src/i2pcontrol/production.rs` — adapter changes only if required by the truthful neutral state API;
- `emissary-cli/src/i2pcontrol/tunnel_manager.rs` — regression-only changes if required, not new action semantics;
- focused tests in existing `emissary-cli/tests/**` and colocated tunnel tests;
- `emissary-cli/tests/m062_dependency_containment.rs` only for exact M115 path bookkeeping if the existing M109 allowlist is insufficient.

No other production path is authorized.

## 8. Work packages

### WP1 — make runtime enablement the composition boundary

Refactor only enough of `setup_router` to make runtime enablement authoritative.

Required behavior:

1. Determine runtime `i2pcontrol_enabled` before constructing any M109-only startup inventory/lifecycle owner that can affect startup behavior.
2. When runtime I2PControl is **disabled**:
   - construct client tunnels through the historical `ClientTunnelManager::new` path;
   - construct server tunnels through the historical `ServerTunnelManager::new` path;
   - do not attach `StartupTunnelLifecycleHandle` to the managers;
   - do not impose M109-only inventory/lifecycle validation on startup tunnel execution;
   - do not create an I2PControl-only server destination observer merely because the feature is compiled.
3. When runtime I2PControl is **enabled**:
   - construct the bounded startup inventory;
   - construct/inject the neutral lifecycle handle;
   - preserve M109 destination observation;
   - use `new_with_lifecycle` for the startup client/server managers.
4. Do not duplicate the configuration parser or create a second runtime-enabled flag.
5. Keep the no-feature build on the historical path.

The preferred implementation is a small composition branch, not feature-matrix duplication throughout the tunnel modules.

### WP2 — replace fabricated contention state with a truthful snapshot

The neutral `StartupTunnelController::state()` contract must never translate lock contention into a semantic state.

Implement the smallest truthful mechanism, for example a controller-owned atomic/snapshot state mirrored at lifecycle transitions, or an equivalently bounded design.

Requirements:

- `Starting`, `Running`, `Stopping`, `Stopped`, and `Failed` are returned only when that is the controller's actual last committed lifecycle state;
- observation must remain cheap and non-blocking enough for synchronous startup inventory reads;
- a transient internal lock must not produce `Starting` unless the lifecycle is actually starting;
- client and server controllers use the same semantic rule;
- generation checks remain authoritative for stale-task completion;
- do not add a new public Proposal status solely to represent internal contention.

If the chosen representation duplicates state between an operation mutex and a snapshot primitive, all writes must be centralized/tested so they cannot diverge silently.

### WP3 — establish a retryable shared startup-client session owner

Replace the manager-seeded one-shot `OnceCell` ownership with the smallest neutral owner that can preserve one-session sharing while active and recover after failure.

Required semantics:

1. **First active member:** create the Yosemite streaming session with the same startup client session options used by the historical shared manager.
2. **Additional active members:** reuse the same session; do not allocate one Yosemite session per startup client.
3. **Creation failure:** report start failure truthfully, retain no fabricated `Running` state, and permit a later `start` to retry session creation.
4. **Stop one of several members:** stop that member's listener/generation but retain the shared session for remaining active members.
5. **Stop final active member:** after the final member generation has completed bounded cancellation, release/drop the shared session so no Yosemite client session remains live solely because the lifecycle handle exists.
6. **Restart sole member:** complete old-generation cancellation/session release before successor session creation.
7. **Restart one member while others remain active:** keep the shared session used by the remaining members and restart only the named member.
8. **Concurrent starts/stops:** serialize shared-session ownership transitions sufficiently to prevent duplicate shared sessions, negative membership, use-after-release, or an old creator publishing a session after all members stopped.
9. **Manager startup:** automatic configured startup still calls the same lifecycle operations, but session creation belongs to the retryable owner rather than a one-time manager pre-seed.

The owner must remain a neutral startup-client implementation detail. It must not implement M110's Proposal `Shared` option or become a general I2PControl session registry.

### WP4 — preserve server lifecycle behavior while fixing state observation

Server runtime execution and destination ownership are otherwise regression-only.

- retain the existing persistent destination material owner;
- retain public-destination-only observation;
- retain bounded cancellation/readiness;
- apply WP2 truthfulness to server state;
- do not change server SAM session semantics, local target behavior, or secret storage.

### WP5 — regression-test the runtime-disabled state that M109 omitted

Add explicit evidence for the three compile/runtime combinations:

1. feature absent;
2. feature compiled + runtime I2PControl disabled;
3. feature compiled + runtime I2PControl enabled.

At minimum, the second case must prove that M109 lifecycle composition is not selected.

Use the smallest practical fixture. Evidence SHOULD include a startup configuration that would exercise the controlled-vs-legacy constructor boundary and a fake/controlled SAM endpoint where necessary.

The test must fail on the M109 implementation head and pass after M115.

### WP6 — shared-session recovery/lifetime regression

With a fake SAM endpoint capable of counting session creation/closure or equivalent deterministic evidence, prove:

- two controlled startup clients use one shared session while concurrently active;
- stopping one retains the session while the other remains active;
- stopping the final client releases the shared session;
- starting again creates a usable successor session;
- an initial shared-session creation failure does not permanently poison later start;
- same-name lifecycle remains serialized;
- no client reports running without a usable listener/session.

Do not require public-network testing for this corrective.

### WP7 — state contention regression

Force state observation while the lifecycle controller is in a contended operation/state transition and prove the returned value is the actual committed state, not a synthetic `Starting` fallback.

Cover both client and server controller state paths or a shared neutral implementation with one controller-specific regression each.

### WP8 — planning and containment reconciliation

Update:

- `plans/registry.md`;
- this roadmap;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- M062 exact planning/production path bookkeeping as needed;
- eventual `plans/closure/i2pcontrol-proposal-170/115-closure.md`.

The registry must make M115 the sole ready handoff. M110-M114 remain blocked/unregistered until M115 closure and their independent primitive gates are satisfied.

## 9. Failure, cancellation, restart, and contention semantics

### Runtime-disable branch

- no M109 lifecycle task/session owner exists when runtime I2PControl is disabled;
- startup manager failure semantics are exactly the historical path's semantics;
- disabled mode cannot fail solely because I2PControl startup inventory/lifecycle construction failed.

### Client session creation

- one shared-session creation attempt at a time;
- cancellation or creation failure does not leave a permanently poisoned owner;
- a failed creator cannot publish a session after ownership moved to zero active members/new generation;
- creation errors remain redacted and do not include destination/session secret material.

### Member stop/restart

- local member runtime is cancelled before membership/session release is considered complete;
- final-member stop releases the shared session only after no other active member owns it;
- restart does not overlap two generations of the same member;
- restart with other members active must not disrupt those members' shared session.

### Contention

- per-name lifecycle remains serialized;
- shared-session membership/session transitions are serialized separately from unrelated member I/O;
- no global lock is held across TCP relay I/O;
- Yosemite session access may retain the existing narrow mutex required for `connect_detached_with_options`, but session-owner bookkeeping must not remain locked across stream relay lifetime.

## 10. Compatibility and migration

No durable schema migration is allowed or required.

Compatibility requirements:

- no-feature Emissary startup is unchanged;
- feature-capable but runtime-disabled Emissary startup follows the historical client/server manager path;
- runtime-enabled M109 API behavior remains available;
- startup configuration files remain authoritative and unmodified;
- control-plane-created TunnelStore definitions are unchanged;
- managed TLS/auth/AddressBook/RouterInfo/ClientServicesInfo behavior is unchanged;
- M095/M105 remain byte/semantically unchanged except test bookkeeping that does not reclassify cells.

The session-lifetime fix may cause a new transient client destination/session to be created after all startup clients have been explicitly stopped and later restarted. That is correct lifecycle behavior for a non-persistent shared client session and must not be represented as M110 `NewDest`/`PersistentClientKey` support.

## 11. Security and anonymity review requirements

Closure must explicitly verify:

- runtime-disabled execution does not instantiate the M109 lifecycle/session owner;
- no new long-lived network activity is created merely by compiling the feature;
- final-member stop does not leave the controlled shared Yosemite client session live indefinitely;
- session errors/logs contain no private destination material;
- server private destination material remains non-debug/non-serialized and only public destination observation crosses into I2PControl;
- no local-target/SSRF/proxy/anonymity boundary changes;
- no M110-M113 security-sensitive option behavior is introduced;
- no dependency/core/util expansion occurred.

## 12. Tests and verification commands

Focused tests must cover WP5-WP7 plus existing M109 lifecycle regressions.

Required verification:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check

cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast

cargo test -p emissary-cli --no-default-features --features i2pcontrol \
  --test m023_startup_inventory \
  --test m033_tunnel_lifecycle \
  --test m061_containment \
  --test m062_dependency_containment \
  --test m095_full_support_matrix \
  --test m105_residual_option_audit \
  --no-fail-fast

cargo test -p emissary-cli --no-default-features --features i2pcontrol \
  --test i2pcontrol_live_runtime -- --nocapture

cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

If a new focused integration target is added for runtime-disable composition, run it explicitly in closure and include its exact result.

The repository's known stable/nightly rustfmt mismatch must be recorded if still present; do not retain unrelated formatting churn.

## 13. Static/containment guards

M115 implementation must amend the cumulative M062 containment evidence only as narrowly as required for the exact authorized paths.

The containment assertion must make clear that:

- the `main.rs` change is runtime composition selection only;
- `client.rs`/`server.rs` changes are neutral startup lifecycle owner corrections;
- Proposal policy remains in `i2pcontrol`;
- no Cargo/core/util/Yosemite/frontend/workflow path is authorized;
- M115 does not expand the M109 exception into a general CLI-tunnel modification allowance.

## 14. Acceptance criteria

M115 closes only if all are true:

1. feature compiled + runtime I2PControl disabled selects historical startup client/server execution and does not instantiate the M109 lifecycle/session owner;
2. runtime-enabled startup lifecycle remains operational for named actions and mixed `All=true`;
3. startup state observation cannot fabricate `Starting` solely because an internal lock is contended;
4. controlled startup clients share exactly one Yosemite client session while multiple members are active;
5. initial shared-session creation failure is retryable by later lifecycle start;
6. stopping the final active controlled startup client releases the shared session;
7. restart cannot overlap same-name generations or duplicate the shared session;
8. server destination/private-key secrecy and M093 invariants remain intact;
9. no production path outside the M109/M115 exact seam changed;
10. no Cargo/Yosemite/core/util/frontend/workflow change exists;
11. M095 remains exactly `224 / 158 / 458` and M105 residual ownership remains 158 cells;
12. focused/broad tests and clippy/checks pass, with any known formatter limitation recorded accurately;
13. planning registry/roadmap/README describe M115 and no longer contain the stale pre-M109 lifecycle statement;
14. closure records no unresolved high/medium M115-scoped finding.

## 15. Stop conditions

Stop implementation and return to planning if any of the following becomes necessary:

- modifying `emissary-core/**` or `emissary-util/**`;
- changing Yosemite source/version or vendoring/path/git overriding it;
- adding/changing Cargo dependencies or `Cargo.lock`;
- implementing M110 `Shared`/client-key semantics to solve this startup-only owner;
- adding a router-global session pool;
- changing startup configuration persistence/ownership;
- widening server local-target/network behavior;
- changing the 158 residual option classifications;
- creating a new frontend or operational control surface;
- upstream interaction of any kind.

A stop condition is evidence that M115 is not the correct owner; it is not permission to widen the patch.

## 16. Closure evidence required

`plans/closure/i2pcontrol-proposal-170/115-closure.md` must include:

- exact implementation commits and changed paths;
- requirement-to-evidence matrix for F1-F5 and every acceptance criterion;
- explicit feature-absent / feature-present-runtime-disabled / feature-present-runtime-enabled evidence;
- shared-session creation count/lifetime/recovery evidence;
- state-contention truthfulness evidence;
- M109 named/`All=true` regression results;
- failure/cancellation/restart/contention review;
- secret/anonymity/security review;
- containment diff and M061/M062 result;
- M095/M105 unchanged evidence;
- exact verification commands/outcomes;
- unresolved findings with severity;
- disposition and M110 readiness decision;
- internal-only attestation.

M115 closure may make M110 eligible for a new readiness review, but it must not mark M110 ready unless M110's independent shared-session/key ownership and accepted-Yosemite capability gates are also satisfied.

## 17. Internal-only attestation requirement

Closure must attest that external Proposal/reference/dependency materials were accessed read-only; all writes stayed within `eggstack/emissary`; no upstream repository, issue, pull request, review, maintainer channel, release, branch/tag, contribution package, merge/adoption request, or submission artifact was created or mutated.
