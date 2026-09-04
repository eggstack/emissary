# M135 Closure — Neutral Live Tunnel Quantity and LeaseSet Reconfiguration Primitive

Status: **closed as complete**

Date: `2026-09-04`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/135-neutral-live-tunnel-quantity-and-leaseset-reconfiguration-primitive.md`

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Planning baseline:

- pre-M135 planning head `517decf733352dfc2bf24ad349c5ab4cf9315742`
- M132 closure head `6618c49a4bcf962a1ee263fa97fa95a3b70f1ad2`
- M133 closure head `517decf733352dfc2bf24ad349c5ab4cf9315742`
- starting matrix `284 apply / 88 blocked_primitive / 468 not_applicable`
- starting matrix SHA-256 `f038521da9cc685bd38dd502f02dcc81f53586c3a8dd83eb3ba5a8827f589f79`

Reviewed head (implementation):

- `3e64f5eda9c66d50c1fcbf066edd86fbe3894c08` plus the workdir diff closed here
  (7 files, `1414 insertions, 39 deletions`; exact paths in §3)

Production-behavior baseline (retained):

- M130 implementation head `fe1a981`
- M130 closure head `a68094e128d2b92f0fd5b350e38512ef6b65cb6b`

Implementation commits:

- One commit on the current branch closing M135 (production + tests +
  M061/M062 guards + planning records). No dependency, lockfile, Yosemite,
  SAM, I2PControl, frontend, or NetDb protocol change.

Pinned authority:

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`
- read-only Java reference snapshot
  `i2p/i2p.i2p@2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`
- M130 runtime/security qualification; M131 residual primitive authority

Current Proposal matrix at closure (mechanically recomputed, unchanged):

- `284 apply / 88 blocked_primitive / 468 not_applicable`
- `095-full-support-matrix.toml` SHA-256
  `f038521da9cc685bd38dd502f02dcc81f53586c3a8dd83eb3ba5a8827f589f79`

## 1. Executive finding

M135 is closed as complete. Emissary now owns a neutral,
destination-scoped live tunnel-quantity/LeaseSet desired-count
reconfiguration primitive with reference-compatible lower-layer behavior
and zero Proposal support promotion.

Realized capability:

- `TunnelPoolConfig.num_inbound/num_outbound` remain immutable base authority;
- `TunnelPool` caches separate `desired_inbound/desired_outbound` targets
  initialized from base and synchronized from a generation-local
  single-slot latest-state control cell;
- future build deficit, standby promotion, and expiry-path promotion use the
  desired target; hop lengths, variances, peer selection, backup quantities,
  and exploratory/participating behavior are untouched;
- lowering the target never deletes or hides excess tunnels; convergence
  happens through normal expiry/failure with no replacement built at or
  above the target; already-dispatched builds may complete and remain until
  normal removal;
- `LeaseSetManager` keeps immutable `base_inbound` plus dynamic
  `desired_inbound`; readiness follows the desired count using only real
  leases; increase enters the existing await-tunnels state; unpublished
  destinations stay unpublished;
- `Destination::set_tunnel_quantity_target` / `restore_tunnel_quantity_target`
  coordinate pool + lease-set targets atomically with explicit failure
  semantics and rollback;
- control is bounded (one slot, latest wins, no queue/task/timer per update),
  generation-local (per-pool cell + generation id, shutdown marks closed and
  wakes), destination-isolated, and holds no lock across build/network I/O.

No SAM idle policy, no I2PControl change, no matrix promotion. Any M095 cell
change under M135 would have been a plan violation; none occurred.

## 2. Requirement-to-evidence matrix

| Plan requirement | Evidence | Result |
|---|---|---|
| §4.1 base vs live target distinct, init to base | `handle.rs` control init from config; `mod.rs` cached desires from config; tests `m135_desired_targets_initialize_to_base`, `m135_desired_initializes_to_base`, `m135_destination_targets_initialize_coherent` | **pass** |
| §4.2 narrow generic atomic target update + restore, no policy vocabulary | `TunnelPoolHandle::set_quantity_target/restore_quantity_target`, `MAX_DESIRED_TUNNEL_QUANTITY = 16` bound, production grep clean (§8) | **pass** |
| §4.3 pool behavior: deficit/standby use desired; no purge/hide; backup/hops/selection/exploratory untouched | deficit fns + `promote_standby_inbound` + expiry promotion use `desired_*`; hop/variance reads unchanged; tests `m135_lowering_changes_deficit_without_mutating_base`, `m135_lowering_preserves_excess_inbound/outbound_state`, `m135_excess_remains_selectable`, `m135_no_replacement_at_or_above_target`, `m135_restore_resumes_deficit`, `m135_backup_targets_unchanged`, `m135_standby_promotion_uses_desired_target` | **pass** |
| §4.4 pending builds: complete, remain, no extra replacement; restore resumes via maintenance wake | deficit includes usable+pending; sync triggers `maintain_pool()` on change; test `m135_no_replacement_at_or_above_target` (excess completion case) | **pass** |
| §4.5 LeaseSet desired count, no fabrication, await-tunnels on increase, unpublished unchanged | `base_inbound/desired_inbound`, `set_desired_inbound_count` reconciliation, `register_inbound_tunnel` readiness on desired; tests `m135_desired_follows_decrease/restore`, `m135_increase_waits_for_real_tunnels`, `m135_decrease_preserves_real_leases`, `m135_unpublished_behavior_unchanged` | **pass** |
| §4.6 destination atomic bridge, no mutable pool/lease-set exposure | `Destination::set/restore_tunnel_quantity_target` with pool-first ordering + rollback; tests `m135_destination_bridge_keeps_targets_coherent`, `m135_destination_rejects_invalid_without_divergence`, `m135_destination_end_to_end_convergence` | **pass** |
| §5 bounded generation-local control: isolation, stale/closed handling, restore-wins, no lock across I/O, no task per update | single-slot cell + generation + closed flag + waker (woken outside lock); tests `m135_quantity_targets_isolated_per_handle`, `m135_pool_targets_are_isolated`, `m135_invalid_and_shut_down_rejected`, `m135_closed_control_is_rejected`, `m135_latest_update_wins`, `m135_coalescing_preserves_latest_restore`; no spawn/queue in new code | **pass** |
| §6 path budget respected | exact realized diff §3; `context.rs` untouched (no channel needed); `tunnel/mod.rs` one re-export seam only; no SAM/I2PControl/Cargo/Yosemite/NetDb/transport/frontend change | **pass** |
| §8 all 20 focused tests | 30 new focused tests (23 `m135_*` pool/lease-set/destination + 7 `m135_*` handle), §4 maps each item | **pass** |
| §9 failure/cancellation/restart/contention | §6 review; generation-local non-persisted targets; shutdown marks closed + wakes; explicit errors; idempotent expiry | **pass** |
| §10 compatibility (no-op without new calls) | full core suite green; no default-behavior change (desired == base until called) | **pass** |
| §11 security review | §8 review; isolation, hop/peer/backup unchanged, no fabricated leases, no secrets in output, no unbounded resource, production grep clean | **pass** |
| §12 broad verification | §5 commands/results | **pass** |
| §13 matrix exactly `284/88/468` | mechanically recomputed, hash agrees | **pass** |

## 3. Production implementation evidence

Exact changed paths (workdir diff vs `3e64f5e`, `1414+/39-`):

- `emissary-core/src/tunnel/pool/handle.rs` — bounded generation-local
  latest-state control cell (`QuantityTargetControl`), `TunnelPoolHandle`
  desired/base getters, atomic `set_quantity_target` /
  `restore_quantity_target`, `MAX_DESIRED_TUNNEL_QUANTITY = 16`,
  `QuantityTargetError::{InvalidQuantity, PoolShutDown}`;
- `emissary-core/src/tunnel/pool/mod.rs` — cached `desired_inbound/outbound`
  + generation + shared cell on `TunnelPool`; deficit, inbound standby
  promotion, and outbound expiry-promotion use desired; hop/variance/backup
  reads unchanged; poll stores waker, syncs target, runs maintenance on
  change, marks closed on shutdown;
- `emissary-core/src/tunnel/mod.rs` — one re-export seam for the two
  handle-owned names so the destination bridge can name them (4+/1-);
- `emissary-core/src/destination/lease_set.rs` — immutable `base_inbound` +
  dynamic `desired_inbound`, getters, `set_desired_inbound_count` with
  await-tunnels reconciliation, readiness on desired with real leases only;
- `emissary-core/src/destination/mod.rs` — narrow
  `set/restore_tunnel_quantity_target` bridge with pool-first ordering,
  rollback, and waker nudge; base/desired getters;
- `emissary-cli/tests/m062_dependency_containment.rs` — new
  `is_authorized_m135_path` budget fn wired into both allow checks;
- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml` —
  three new `core_owner_hooks` entries + evidence (handle, destination,
  lease_set).

Not changed (per §6): `tunnel/pool/context.rs`, `sam/**`,
`emissary-cli/src/i2pcontrol/**`, Yosemite, manifests/lockfile, NetDb,
transport/peer selection, frontend/startup/config.

### 3.1 Direct reference freeze (WP1)

Frozen before code from the pinned snapshot and roadmap §3:

| # | Behavior vector | Pinned source |
|---|---|---|
| 1 | quantity decrease with excess live tunnels: settings replaced, live tunnels stay usable, build demand follows new quantity | `router/.../tunnel/pool/TunnelPool.java` `setSettings()` + build-demand computation; `TunnelPoolManager.java`; roadmap §3.2 |
| 2 | quantity decrease with pending builds: dispatched builds may complete; no extra replacement at/above target | same pool build algorithm (§3.2); M135 preferred rule §4.4 |
| 3 | no replacement above target | build deficit from current settings quantity (§3.2) |
| 4 | restore to configured quantity via reconfiguration, no pool recreation | `I2CPMessageProducer.updateTunnels(session, 0)` (§3.1); router `handleReconfigureSession` (§3.2); M135 §3.5 |
| 5 | LeaseSet wanted count follows current inbound quantity: `wanted = min(current inbound quantity, MAX_LEASES)`; rebuild on normal inbound lifecycle | `TunnelPool.locked_buildNewLeaseSet()` (§3.4) |
| 6 | reconfiguration carries inbound+outbound atomically (`ReconfigureSessionMessage`, `setInbound/OutboundSettings`) | `I2CPMessageProducer.java`, `ClientMessageEventListener.java` (§3.1–3.2) |

M135 implements this shape; any contradictory newer behavior at the same
snapshot would have stopped the plan per §3 — none was found.

### 3.2 Before/after target traces

- before any call: `(base, desired) == config` on handle, pool, and
  destination (`m135_*_initialize_*` tests);
- `set(1, 2)` on base `(3, 3)`: handle/pool report desired `(1, 2)`, base
  stays `(3, 3)`; inbound deficit `3 -> 1`, outbound `3 -> 2`;
- `restore()`: desired returns to `(3, 3)`; deficits return to base.

### 3.3 Excess-tunnel lifecycle trace

- 3 live inbound records + desired lowered `3 -> 1`: map length stays 3,
  selector still yields a tunnel, deficit `0` (tests
  `m135_lowering_preserves_excess_inbound`, `m135_excess_remains_selectable`,
  `m135_no_replacement_at_or_above_target`);
- completing a pending build above target (2 live vs desired 1): deficit
  stays `0`; no further build enqueued.

### 3.4 LeaseSet desired-count/convergence trace

- base 3, three real leases registered → `AwaitingLeaseSet`; decrease to 1:
  all 3 records retained, state stays `AwaitingLeaseSet` (no deletion, no
  synthetic publish);
- 1 real lease, desired raised `2 -> 3`: state `AwaitingTunnels` until the
  3rd real lease arrives, then `AwaitingLeaseSet` (never full-count early);
- unpublished manager: desired updates retained, state pinned `Inactive`.

### 3.5 Bounded control/generation evidence

- each handle/pool pair captures a unique generation; two handles never share
  a cell (isolation tests);
- burst `set(1,1) → set(2,1) → restore()` before the pool polls: pool observes
  only the newest pair; a second sync without new input reports no change;
- `shutdown()` marks the cell shut down and wakes; later sets return
  `PoolShutDown` and the pool keeps its last synchronized target;
- new code spawns no task, creates no queue/timer per update; the control
  lock is held only for short copies and the waker is woken after release.

## 4. Focused tests

30 new deterministic tests, all passing:

- handle (7): init-to-base, lowering preserves base, restore, invalid/shut-down
  rejection, per-handle isolation, latest-wins, no policy vocabulary;
- pool (12): init, deficit-without-base-mutation, inbound excess preserved,
  outbound state preserved, excess selectable, no replacement at/above target
  (incl. post-completion), restore resumes deficit, backup unchanged, standby
  promotion on desired, client/exploratory isolation, shut-down rejection,
  coalescing;
- lease-set (6): follows decrease/restore, increase waits for real tunnels,
  decrease preserves leases, unpublished unchanged, no policy vocabulary;
- destination (5): coherent init, atomic bridge + restore, invalid without
  divergence, cross-destination isolation, end-to-end decrease/expiry/restore.

Plan §8 items 1–20 map: 1→init tests; 2→deficit test; 3→inbound excess;
4→outbound state; 5→selectable; 6→no-replacement; 7→post-completion;
8→restore; 9→backup; 10→standby; 11→pool isolation (exploratory untouched);
12→handle/destination isolation; 13→follows decrease; 14→follows restore;
15→waits for real tunnels; 16→preserves leases; 17→unpublished; 18→shut-down
rejection; 19→latest-wins/coalescing; 20→vocabulary tests + production grep
(§8, clean on added lines; remaining hits are pre-existing
`ChannelError::Closed` / `GetClosestFloodfills` / `TunnelManager` owner
names, untouched).

## 5. Verification executed

At the implementation head with the workdir diff applied:

| Command | Result |
|---|---|
| `cargo check -p emissary-core` | **pass** |
| `cargo test -p emissary-core --no-fail-fast` | **pass**: `1105 passed, 2 ignored (5 suites)` |
| `cargo test -p emissary-core --lib -- m135` | **pass**: `30 passed` |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo check -p emissary-cli --no-default-features` | **pass** |
| `cargo check` (workspace) | **pass** |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast` | **pass**: `34 passed (4 suites)`; matrix `284/88/468`, hash `f03852…f79` |
| `cargo clippy -p emissary-core --all-targets -- -D warnings` | **pass**: no issues |
| `cargo fmt --all -- --check` | **evidence only**: pre-existing stable/nightly drift repo-wide (nightly-only options unavailable under stable); our added lines follow the nightly config; no unrelated normalization performed |
| `git diff --check` | **pass** (no whitespace errors) |

`cargo test -p emissary-cli --no-default-features --features i2pcontrol`
(full) and `i2pcontrol_live_runtime` were not re-run: no I2PControl
production path changed and the M130 live-runtime qualification is inherited,
not re-claimed. The containment/matrix guards plus core suites cover the
affected surface.

## 6. Invariant review

- exact pinned names/types/presence preserved; zero matrix change (§13);
- no fabricated support: no SAM/I2PControl/Proposal path added; dormant
  Yosemite fields untouched;
- every `apply` cell still changes real behavior (zero promotions);
- unsupported values fail before effect (`InvalidQuantity` before any state
  change; ordering + rollback prevent divergent pairs);
- no direct-clearnet, outproxy, trusted-peer, or Streamr isolation change;
- loopback confinement, bounded admission/tasks/timers, transactional
  lifecycle, last-known-good preserved;
- no lock across network/build I/O (short control copies only);
- secret/key/path redaction preserved (no new log/event/Debug payload);
- no LeaseSet crypto/scope change; no publish of nonexistent tunnels;
- feature/runtime isolation, no base-method parity, no frontend coupling.

## 7. Failure, cancellation, restart, and contention review

- desired targets are generation-local and never persisted; restart
  re-initializes from base config;
- pool shutdown marks the cell shut down and wakes any waiter; handle
  shutdown does the same; further updates are explicit `PoolShutDown` errors;
- failed updates change nothing (validation first; pool-first ordering with
  rollback on the unreachable second failure);
- stale generations cannot reach replacement pools (per-pool cell +
  generation check; sync ignores mismatches);
- overload coalesces deterministically (single slot, newest wins; restore
  cannot be lost);
- excess expiry/failure stays idempotent under target changes;
- LeaseSet transitions stay owned by `LeaseSetManager`; no second
  publisher/timer created;
- shared-session registry and pool-build I/O locking discipline unchanged.

## 8. Migration and compatibility review

- no public API version, method, tunnel type, or action change;
- no durable-store migration; no SAM/I2PControl wire change;
- without calling the new seam, behavior is equivalent to the M130-qualified
  runtime (desired == base; same quantities, backups, lengths, builds,
  publication);
- new APIs are additive and `#[allow(dead_code)]`-marked where M136 is the
  intended consumer; no existing caller behavior changed;
- rollback: reverts to base desires by construction (`restore_*`).

## 9. Security review

- per-destination ownership: separate control cell per pool; isolation tests
  for handles, pools, and destinations;
- exploratory/participating pools unreachable from the destination seam (no
  registry; NetDb-owned pools never observe client updates — isolation test);
- reduction cannot force zero-hop/direct-clearnet: hop lengths, variances,
  peer selection untouched;
- backup capacity never promoted beyond desired active target (backup reads
  unchanged; standby promotion gated on desired);
- LeaseSet never advertises nonexistent tunnels (real-lease-only readiness;
  increase awaits tunnels; decrease retains records without publishing);
- no key/secret/path data in new logs/debug/events (debug strings tested
  clean);
- no new unbounded queue/task/timer;
- changed production content contains no Proposal/I2PControl policy
  vocabulary (added-line grep clean; pre-existing neutral `Closed`/
  `ClosestFloodfills`/`TunnelManager` owner names untouched).

## 10. Documentation and operations

- machine authorities unchanged: `095-full-support-matrix.toml`
  (`284/88/468`, `f03852…f79`), `105-residual-option-audit.toml`,
  `110-completion-ledger.toml`;
- support docs need no change (no support claim changed);
- static guards green with exact-budget amendments: M061 manifest gains the
  three new `core_owner_hooks` paths + evidence; M062 gains
  `is_authorized_m135_path` (5 production paths incl. the one-line
  `tunnel/mod.rs` re-export seam + self + closure/plan/registry/roadmaps);
- operational impact: none (no config, metric, diagnostic, or restart change).

## 11. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | `cargo fmt --all -- --check` reports pre-existing stable/nightly drift repo-wide (incl. untouched regions of changed files) | none on behavior; evidence-only | record, do not normalize unrelated source |
| low | new primitive has no SAM/I2PControl consumer yet by design | no reduction support until M136 | M136 consumes via the destination bridge |

No high/medium correctness defect remains in the primitive.

## 12. Roadmap disposition and M136 readiness

- M135 is **closed as complete** with matrix `284/88/468` unchanged.
- M136 registration gate (plan `136-*.md` §2) is fully satisfied:
  1. destination-scoped update changes desired inbound/outbound — proven;
  2. base config immutable — proven;
  3. deficit + standby promotion use desired — proven;
  4. excess remains valid until normal removal — proven;
  5. LeaseSet desired count follows inbound without fabrication — proven;
  6. restore deterministic — proven;
  7. bounded generation-local control — proven;
  8. M061/M062 green — proven;
  9. matrix exactly `284/88/468` — proven;
  10. no unresolved high/medium defect — proven.
- **Decision: M136 is dependency-ready and may register.** M136 remains
  unregistered until its own registration step flips its status; this closure
  authorizes that flip and no other production work.
- M137 stays deferred until M136 closes; M134/M138 stay future until M137
  closes. Other M131 residual clusters remain unregistered. M132/M133 closures
  are immutable history.

## 13. Registry updates

Applied alongside this closure:

- `135-*.md` plan: Status `ready / registered` → `closed as complete` with
  closure link;
- `plans/registry.md`: M135 → closed as complete; handoff M135 → M136
  dependency-ready/unregistered (registers on its own step); matrix retained;
  M135 added to recently-closed;
- `plans/implementation/.../README.md`: handoff M135 → M136 ready gate passed;
  dependency graph updated;
- session-lifecycle + full-support roadmaps: M135 complete, zero promotions;
  M136 gate satisfied;
- `136-*.md` / `137-*.md`: status notes M135 closure satisfies the M136 gate
  (registration itself is a separate step).

## Internal-only / read-only-upstream attestation

- external sources (Proposal 170 text, pinned Java snapshot paths, I2CP
  semantics via roadmap freeze) were accessed read-only for evidence;
- no upstream or third-party repository, issue, pull request, discussion,
  review, or maintainer channel was opened, drafted, updated, commented on,
  or contacted;
- no commit, branch, tag, patch, release, or artifact was pushed to any
  upstream remote. All writes are internal to `eggstack/emissary` on the
  current branch;
- no upstream review, approval, feedback, adoption, or merge was requested;
- no upstream contribution package, patch series, or submission checklist was
  prepared;
- violation would invalidate this closure per `plans/003-planning-process.md`
  §11; no such violation occurred.

(End of file)
