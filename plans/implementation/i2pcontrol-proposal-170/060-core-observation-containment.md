# M060 — Core Observation Seam Consolidation and Containment

Status: closed

Planning baseline: `ed17fe7` — M059 implementation head; M059 closure accepted in the following planning commit

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`

Hard dependencies:

- M058 accepted containment ledger and core candidate budget;
- M059 accepted closure proving original CLI/runtime containment and no core changes during M059;
- no unresolved M058 `uncertain` core path may be modified.

Milestone class: corrective implementation / security containment

Applicable authority:

- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`;
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`;
- `plans/implementation/i2pcontrol-proposal-170/037-containment-boundary-reduction.md`;
- accepted M058 ledger/closure;
- accepted M059 closure.

## 1. Bounded objective

Reduce and consolidate the `emissary-core` delta required by the supported I2PControl/Proposal 170 surface while preserving truthful live observations and leaving all router/protocol behavior unchanged.

M060 is not a request to make core “I2PControl-aware.” It is the opposite: core may retain only generic read-only inspection and the smallest owner-local passive hooks required for facts that cannot be truthfully reconstructed at a higher layer.

The implementation must prefer, in this order:

1. revert an unnecessary core change to upstream;
2. derive the same authoritative fact from an already-retained higher-level owner;
3. consolidate several lower-level hooks into one neutral owner-level seam;
4. retain a lower-level hook only when exact truth/order/bounds cannot otherwise be preserved.

A smaller file count is desirable but not normative. The normative result is **minimum justified core delta without semantic degradation**.

## 2. Expected core target groups

The accepted M058 ledger is authoritative. At planning time, current fork/upstream comparison shows these likely groups.

### 2.1 Generic inspection/router plumbing

- `emissary-core/src/error/mod.rs`;
- `emissary-core/src/events.rs`;
- `emissary-core/src/inspection.rs`;
- `emissary-core/src/lib.rs`;
- `emissary-core/src/primitives/router_identity.rs`;
- `emissary-core/src/router/context.rs`;
- `emissary-core/src/router/mod.rs`;
- `emissary-core/src/runtime/mod.rs`;
- `emissary-core/src/subsystem/mod.rs`.

Expected legitimate responsibilities include neutral snapshot DTOs/handles and owner aggregation. Constructor/signature propagation that touches unrelated callers solely to carry an optional observer is a primary candidate for reduction.

### 2.2 SAM/I2CP lifecycle observation

- `emissary-core/src/i2cp/socket.rs`;
- SAM module/parser/pending/session/socket/streaming files named by M058.

M037 already established that core should not own I2PControl aggregation/recovery policy. M060 must verify that subsequent work did not reintroduce it and should reduce deep lifecycle-threading where a smaller authoritative hook suffices.

### 2.3 Transport observation

- `emissary-core/src/transport/mod.rs`;
- NTCP2 module/session paths;
- SSU2 message/peer-test/relay/session/socket paths named by M058.

These are high-sensitivity audited protocol paths. Retention requires exact evidence that the observed value is only authoritative at that hook and cannot be collected at `TransportManager` or another already-modified transport owner.

### 2.4 Tunnel observation

- `emissary-core/src/tunnel/mod.rs`;
- `emissary-core/src/tunnel/pool/mod.rs`;
- `emissary-core/src/tunnel/transit/mod.rs`.

Retained observations must be limited to bounded pool/participation/queue facts already consumed by accepted RouterInfo behavior. M060 must not reintroduce request-driven transit-15s sampling or new tunnel metrics.

## 3. Required invariants

1. Router algorithms and network behavior are unchanged.
2. Peer selection, profile scoring, NetDB discovery/query, tunnel selection/build/routing, transport handshake/retransmission/congestion, cryptography, LeaseSet, and I2NP behavior are unchanged.
3. No currently unavailable RouterInfo source becomes available.
4. RouterInfo remains 37 available / 1 neutral / 5 unavailable.
5. Existing supported RouterInfo fields retain exact source truthfulness and wire semantics.
6. ClientServicesInfo retains complete/incomplete/fail-closed semantics and accepted SAM/I2CP observations.
7. Supported TunnelManager client/server lifecycle remains unchanged.
8. Core contains no Proposal 170 selector names, JSON-RPC types, TunnelManager policy, AddressBook administrative policy, or control-plane support classification.
9. No socket, mutable session/tunnel/transport/router handle, command channel, private/session key, LeaseSet private material, or message payload crosses an inspection boundary.
10. All collections/history are bounded; no lock crosses `.await`, sleep, network I/O, or serialization.
11. Observer/inspection failure never changes underlying router/protocol lifecycle.
12. No new global event bus, generalized observer framework, polling loop, persistent metrics store, network probe, or I2PControl-specific background task.
13. No new core production path may be added beyond the accepted M058/M060 budget without stopping for replanning.
14. No upstream interaction is authorized.

## 4. Scope and changed-path budget

Only core paths tagged for M060 by the accepted M058 ledger may be modified.

Always permitted adjunct paths:

- `emissary-cli/src/i2pcontrol/**` only to adapt consumers to a smaller neutral core seam;
- focused `emissary-core` and `emissary-cli/tests/**` regressions;
- containment manifests/closure/docs required by this milestone.

M060 must **not** reopen original CLI/runtime containment. Paths changed by M059 outside `i2pcontrol` are frozen unless M060 exposes a direct incompatibility with the core seam; such a case is a stop condition requiring explicit replanning.

Explicitly prohibited regardless of ledger unless a new plan supersedes this one:

- `emissary-core/src/crypto/**`;
- destination/LeaseSet behavior changes;
- NetDB protocol/query/discovery changes;
- I2NP behavior changes;
- new tunnel data planes;
- `.github/**` and release machinery.

## 5. Target core architecture

### 5.1 Neutral inspection DTOs and handles

Prefer one neutral `inspection` vocabulary for bounded snapshots that can be cloned/read without exposing mutable owners. DTOs should express router facts, not control-plane field names.

Good examples:

- peer identifier and sanitized transport stats;
- bounded known-peer info required by existing inspection consumers;
- tunnel pool counts/details;
- queue depths;
- reachability/testing facts already owned by transport;
- cumulative/recent metrics already canonically maintained by router owners.

Bad examples:

- `i2p.router.*` selector names;
- JSON/base64 formatting policy;
- `Unavailable` decisions tied to Proposal 170;
- control-plane generations/revisions;
- administrative tunnel definitions.

Do not force every fact through a giant `CoreSnapshot` if that increases copying/locking or stale-state risk. Small purpose-specific neutral handles are acceptable where they reduce owner propagation.

### 5.2 Passive lifecycle hooks

A lifecycle hook is allowed only when snapshots alone cannot reconstruct authoritative state, especially SAM/client-service lifecycle.

Hooks must be:

- optional/passive;
- bounded and non-blocking in owner poll paths;
- sanitized before publication;
- incapable of controlling owner lifecycle;
- minimal in event variants/fields;
- absent/no-op by default.

I2PControl owns aggregation, overflow/incomplete policy, public bounds, and serialization.

### 5.3 Transport owner consolidation

For each NTCP2/SSU2 lower-level modification, ask whether the required statistic can instead be updated at a common transport owner where the same event is already visible.

Consolidation is allowed only if the higher owner receives all events necessary to preserve:

- peer identity;
- active/inactive lifetime;
- direction/transport identity where required;
- byte/message counters or other accepted stats with the same semantics;
- v4/v6 testing/status transitions where already accepted.

If higher-level collection would infer a state rather than observe it, retain the owner-local hook and document it.

No instrumentation may be added merely to obtain unavailable network-error reasons or transit-15s data.

### 5.4 Tunnel owner consolidation

Prefer pool/transit owner snapshots over hooks in individual tunnel message/data paths. Existing queue/pool facts may remain at the manager owning them.

The core must not know RouterInfo field groupings or list/map serialization shape.

## 6. Ordered work packages

### WP1 — Freeze accepted core budget and pre-change semantics

Read the M058 ledger and M059 closure. Produce a working table of each authorized core path with:

- changed symbol/hunk;
- current consumer;
- accepted field/service behavior it supports;
- proposed disposition: revert, consolidate, retain;
- exact regression fixture/test.

Run pre-change focused tests. Record pre-existing failures.

### WP2 — Remove obsolete/dead core scaffolding

Start with changes whose consumer disappeared in M054/M055/M059 or which are no longer used after earlier corrective passes.

Examples to look for, without assuming they exist:

- unused error-state plumbing after network-error demotion;
- transit sampling/history state no longer consumed;
- exports or constructor parameters retained after policy moved upward;
- duplicate snapshot fields superseded by a live handle.

Revert to upstream where possible rather than replacing dead code with another abstraction.

### WP3 — Consolidate generic router/inspection plumbing

Reduce constructor/context/runtime/subsystem propagation where an inspection handle can be constructed at the canonical owner and passed only to the composition point that needs it.

Avoid changing public core APIs unrelated to the feature. If an optional observation parameter caused widespread test/example signature churn, consider a builder/optional setter/owner-issued handle only if it actually reduces the total audited surface and does not introduce hidden mutable state.

Do not perform a general API redesign.

### WP4 — Reconcile SAM/I2CP observation hooks

Review every M058 SAM/I2CP path.

Target end state:

- lifecycle owner emits the smallest sanitized events;
- aggregation/recovery maps live outside core;
- parser/request handling is untouched unless it uniquely knows an accepted lifecycle fact;
- connection/session/socket internals are not modified just to thread policy types.

Where several files only propagate an observer reference, seek a higher owner or small optional context shared by the existing subsystem. Do not trade a few parameter changes for a new generic framework.

Retain exact complete/incomplete recovery behavior.

### WP5 — Reconcile transport hooks

For each NTCP2/SSU2 path:

1. identify the accepted observation it writes;
2. locate all equivalent state transitions at higher owners;
3. build a before/after fixture or focused unit test;
4. move accounting upward only if semantics are exact;
5. revert the lower-level path to upstream;
6. otherwise retain the lower hook and record necessity.

Particular scrutiny is required for changes in message/data, relay, pending/active/terminating session, and socket files because they are deep protocol paths.

No timing/retry/handshake/congestion behavior may be touched around the observation hook.

### WP6 — Reconcile tunnel hooks

Apply the same owner analysis to tunnel/pool/transit paths. Prefer direct bounded owner snapshots.

Do not add a rolling sampler or data-plane instrumentation for the unavailable transit-15s selector.

### WP7 — Minimize public exports and dependencies

After consolidation:

- remove core public exports used only by deleted hooks, unless part of an existing documented public contract;
- remove unused types/imports;
- do not perform unrelated API cleanup;
- do not add dependencies.

### WP8 — Update containment evidence

Update the M058 ledger with final M060 disposition metadata without rewriting its baseline classification history, or create an M060 result manifest referenced by closure.

Record:

- paths reverted fully to upstream;
- paths with reduced hunks;
- retained paths and exact necessity rationale;
- no-new-core-path proof.

### WP9 — Closure

Create `plans/closure/i2pcontrol-proposal-170/060-closure.md` with:

- exact baseline/head;
- before/after core changed-path set;
- per-path disposition table;
- accepted behavior regressions;
- security/performance review;
- any retained deep protocol hooks and why they cannot move upward;
- confirmation that the 37/1/5 source matrix is unchanged;
- internal-only/no-upstream attestation.

M061 becomes ready only after this closure is accepted.

## 7. Failure, cancellation, restart, and contention semantics

### Observation failure

- A failed passive observation must never fail or alter the underlying peer/session/tunnel/router lifecycle.
- Where I2PControl requires a complete view, publication loss/overflow continues to mark the control-plane snapshot incomplete and fail closed rather than serving partial state as current.
- Snapshot read failure maps to the accepted sanitized unavailable/internal error behavior in `i2pcontrol`, not protocol behavior.

### Cancellation

Core hooks do not own cancellation tokens or task handles for router subsystems. Removing/replacing a hook must not change task shutdown ordering.

### Restart

No new durable core observation state is created. On restart, live state is reconstructed from canonical owners/events as before; stale observer state must not survive as authoritative router state.

### Contention

- no lock across `.await`, network I/O, sleep, serialization, or callback into arbitrary user code;
- no unbounded queue;
- no synchronous work in packet/session hot paths beyond the accepted minimal observation update;
- consolidation should decrease or hold steady hot-path overhead.

## 8. Compatibility and migration

No protocol/wire/persistence/config migration.

Core public API changes are permitted only to remove/narrow fork-introduced inspection machinery after all internal consumers migrate. Do not opportunistically alter unrelated upstream-compatible APIs.

Examples/tutorials/tests changed solely because of a fork-introduced constructor signature should be restored toward upstream if the signature can be narrowed safely; this is a useful containment signal, not an independent goal.

## 9. Security review requirements

For every retained core seam verify:

- no secret/private key or raw destination private material;
- no live socket/stream/session handle;
- no mutable manager/storage/router handle;
- no command sender/receiver/control capability unless the underlying supported tunnel backend requires a narrow capability explicitly approved elsewhere;
- no raw message payload;
- bounded collections and sanitized text;
- no Proposal 170/I2PControl terminology;
- no change to router decisions or timing semantics;
- no new dependency.

For every deep transport/tunnel/SAM path retained, closure must explicitly answer: “Why is this fact not truthfully observable at a higher already-modified owner?”

An unanswered question blocks closure.

## 10. Focused tests

Required categories, using existing exact test names where present:

- core inspection snapshot/handle unit tests;
- SAM observer absent-path and complete/incomplete recovery tests;
- I2CP ClientServicesInfo lifecycle observation;
- transport active-peer inventory/stats/status/testing fixtures;
- tunnel pool/participating/queue snapshot fixtures;
- RouterInfo truthfulness and production adapter tests;
- default/no-feature core behavior;
- no live/secret type crossing containment guards.

Do not create tests for unavailable news/ban/error/transit-15s functionality beyond retained truthfulness guards.

## 11. Verification commands

Minimum local matrix, adjusted to actual test target names if repository naming changed:

```bash
cargo check -p emissary-core
cargo test -p emissary-core sam
cargo test -p emissary-core transport
cargo test -p emissary-core tunnel
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test client_services_live
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test router_info_truthfulness
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m037_containment
cargo clippy -p emissary-core --all-targets -- -D warnings
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

Also run every regression named for an M060 path in the accepted M058 ledger.

Changed-path checks:

```bash
git diff --name-only <M060_BASE>..HEAD
git diff --name-only 9b43484a21d5a1291c4881cdae62a36c527f8c0f..HEAD -- emissary-core
```

No hosted CI or new verification infrastructure is required.

## 12. Documentation and static guards

Update `docs/i2pcontrol/inspection-architecture.md`, `router-info-source-map.md`, `client-services.md`, and `security.md` only where final core ownership changes.

Do not update the historical M037 manifest to pretend it described later RouterInfo work. M061 will create/enforce the current final containment manifest.

M060 may add focused regression assertions, but the final cross-workstream path guard belongs to M061.

## 13. Acceptance criteria

M060 may close only when all are true:

1. Every modified core path is within the accepted M058 M060 budget.
2. No new core production path was introduced.
3. Every M058 `candidate-revert` core path is reverted or closure records exact regression evidence requiring reclassification.
4. Every `candidate-consolidate` group is either consolidated to a smaller/higher owner or closure proves why consolidation would lose truth/order/bounds.
5. Obsolete error/transit/other superseded observation scaffolding is absent.
6. Core contains no Proposal 170 selector/wire/admin/support policy.
7. SAM/I2CP core retains only minimum sanitized owner-local lifecycle facts; aggregation/recovery/public bounds stay outside core.
8. Transport deep protocol hooks are reduced where possible; every retained one has canonical-owner necessity evidence.
9. Tunnel core retains only bounded owner snapshots/hooks required by accepted supported fields; no transit-15s sampler exists.
10. Router/protocol behavior, timing, task ownership, cancellation, and persistence are unchanged.
11. No live/secret/mutable control objects cross inspection boundaries.
12. Default/no-feature and I2PControl-feature focused tests pass except explicitly recorded unrelated pre-existing failures.
13. Accepted RouterInfo source matrix remains 37/1/5 and M051 remains blocked.
14. No new dependency, event framework, probe, sampler, persistent metric store, CI/release machinery, or unsupported data plane was added.
15. Closure contains a before/after core path set and exact rationale for every retained path.
16. No upstream interaction occurred.

## 14. Stop conditions

Stop and create a new corrective disposition rather than broadening M060 if:

- a necessary edit falls outside the accepted core budget;
- preserving a supported observation requires changing router/protocol behavior;
- higher-level consolidation would require inference/fabrication rather than authoritative observation;
- moving a SAM/transport/tunnel hook requires an unbounded queue or general event framework;
- a current supported field is discovered to have no authoritative owner;
- a persistence/wire/config migration appears necessary;
- an unavailable Proposal 170 source is proposed as justification for new instrumentation;
- a new external spec revision materially changes requirements;
- any upstream write/review/submission action is proposed.

## 15. Expected closure disposition

Successful M060 closure should establish that the remaining core delta is the minimum justified neutral observation surface at this repository state, with no behavior expansion, and hand M061 a frozen final path set for independent reclosure/static enforcement.
