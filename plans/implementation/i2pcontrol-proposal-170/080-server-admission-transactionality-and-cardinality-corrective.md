# M080 — Server Admission Transactionality and Cardinality Corrective

Status: closed; corrective history — M083 closes remaining current-head capacity and expiry-index defects

Closure: `plans/closure/i2pcontrol-proposal-170/080-closure.md`.

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`.

Original implementation/closure:

- M074: `plans/implementation/i2pcontrol-proposal-170/074-server-admission-and-rate-limit-hardening.md`;
- M074 closure: `plans/closure/i2pcontrol-proposal-170/074-closure.md`.

Planning production baseline: `1618de172e7a78a193fc1bb117af269f31174030`.

## 1. Objective

Correct the shared accepted-server admission implementation so rejected connections cannot mutate durable accounting state, all expiry/accounting structures are hard bounded, authenticated peer keys use the canonical I2P Destination identity rather than a 64-bit general-purpose hash, and default/configured rate semantics cannot silently create a small fixed peer-table choke point that is exhausted long before tracked windows expire.

This is a narrow corrective to M074. It must preserve the accepted-stream architecture, reference-scale rate controls, per-peer fairness, prompt overload rejection, and I2PControl containment that M074 established.

## 2. Independent findings that reopen M074

### 2.1 HIGH — aggregate-rate rejection leaks a new peer record

At baseline, `ServerAdmissionState::try_acquire`:

1. derives a `PeerKey`;
2. reaps expired records;
3. checks table/global capacity;
4. inserts a previously unseen peer record;
5. checks peer limits;
6. checks aggregate limits;
7. only on success records counters and queues expiry.

When a new peer is inserted and then rejected by the aggregate rate check, the peer record has no accepted event and no expiry-queue entry. `reap()` only processes keys reached through the expiry queue. Repeating aggregate-rejected attempts with fresh Destinations can therefore fill the peer table with zero-active, unexpiring records and force `PeerStateCapacity` for all previously unseen peers until runtime restart.

M074 closure explicitly claimed bounded churn-safe state, so this defect invalidates that closure disposition.

### 2.2 MEDIUM-HIGH — expiry bookkeeping is not itself hard bounded

`State.expirations` is a `VecDeque<ExpiryEntry>` and new entries are appended on accepted activity and again when leases drop. The peer map is bounded, but stale/superseded expiry entries may accumulate independently of peer cardinality.

The invariant is stronger than "the primary map is bounded": every attacker-influenced accounting structure must have a documented bound tied to configured capacity.

### 2.3 MEDIUM-HIGH — fixed 4096 peer capacity conflicts with long default windows

The default policy tracks per-peer day state while accepting up to 50 aggregate connections per minute. A stream of fresh authenticated Destinations can therefore consume 4096 peer slots substantially before a 24-hour peer window expires. Fail-closed capacity protects memory but converts the fixed table into a predictable deny-new-peer state.

The correction must preserve exact rate semantics while making capacity coherent with the maximum distinct peer arrivals permitted by the configured aggregate policy over the longest enabled peer-tracking window, subject to an explicit hard memory ceiling.

### 2.4 LOW-MEDIUM — peer accounting uses `DefaultHasher`/64-bit keys

Security accounting currently reduces trusted Destination text to eight bytes through Rust's general-purpose `DefaultHasher`. I2P Destinations already have a canonical cryptographic identity hash. Admission should key on that canonical fixed-size identity rather than an unspecified 64-bit hash.

## 3. Why prior verification missed these findings

M074 tests verified:

- global/per-peer concurrency;
- per-peer and aggregate windows;
- table-full rejection;
- expiry after a successful admission;
- lease release.

They did not exercise the state transition "new identity inserted -> later aggregate rejection" and did not assert that every denial path leaves table/expiry sizes unchanged. They also asserted a fixed peer-table bound without checking whether configured/default arrival rates can exhaust that bound before the longest retained window expires. Expiry tests checked correctness, not queue cardinality under repeated refresh/release.

M080 must add regression tests at those exact missed transitions.

## 4. Hard invariants

- production changes remain under `emissary-cli/src/i2pcontrol/**` except a narrowly justified test/dependency metadata correction;
- no `emissary-core/**` production change;
- no router/SAM protocol extension;
- trusted identity originates only from Yosemite/SAM accepted streams;
- admission denial occurs before protocol handler/local-target allocation;
- a denied attempt must not increase active counts, peer records, accepted-rate counters, or expiry-index cardinality;
- no active/throttled peer state is evicted merely to admit a new untrusted identity;
- every attacker-influenced collection has an explicit finite bound;
- no lock is held across network I/O, task execution, sleeps, joins, or target connect;
- monotonic time remains authoritative;
- no random/fixed rejection delay or jitter is added;
- private destination material and full peer Destination text remain absent from diagnostics;
- zero continues to mean unlimited for Proposal 170 rate fields where already established;
- unsupported/underspecified Proposal 170 fields remain fail-before-allocation rejects.

## 5. Required production changes

### 5.1 Make admission transactional

Refactor `ServerAdmissionState::try_acquire` so every rejection path is side-effect free except bounded internal housekeeping such as reclaiming already-expired state.

Required ordering for a previously unseen peer:

```text
validate/derive canonical PeerKey
-> reap bounded expired state
-> check global concurrency
-> check peer-state representability/capacity
-> evaluate new-peer peer-rate eligibility (zero prior count)
-> evaluate aggregate-rate eligibility
-> reserve map capacity if needed
-> commit peer record + accepted counters + active counts + expiry index atomically under the lock
-> return AdmissionLease
```

Equivalent transactional/rollback logic is acceptable, but tests must prove rollback for every denial after tentative allocation. Prefer check-before-mutation because it is easier to reason about and audit.

For existing peers, failed peer/aggregate checks must likewise leave counts and expiry metadata unchanged.

### 5.2 Canonical cryptographic peer identity

Replace the eight-byte `DefaultHasher` key with a fixed-size canonical I2P Destination identity derived from the structurally validated full Destination. Reuse existing Emissary/I2PControl primitives where possible, for example `emissary_core::primitives::Destination::parse(...).id()` through an I2PControl-local helper.

Target representation: a 32-byte Destination hash or an equivalent canonical fixed-size cryptographic identity already produced by the core parser.

Do not add a new hashing dependency merely for this plan if the repository already exposes the canonical Destination ID.

The accepted-stream boundary should reject malformed/non-canonical remote Destination text before it can enter admission/accounting. Update fake-SAM fixtures to use structurally valid test Destinations rather than weakening production validation to preserve placeholder strings.

### 5.3 Bound the expiry index, not only the peer map

Replace append-only stale expiry bookkeeping with a structure whose live cardinality is O(peer capacity), with at most one authoritative expiry registration per peer/accounting record.

Acceptable designs include:

- an indexed expiry map keyed by peer with an ordered time index that supports removing/replacing the prior deadline;
- a bounded slot/slab plus ordered expiry index;
- another standard-library/I2PControl-local structure that proves the same finite bound.

Avoid a `BinaryHeap`/queue design that merely tags generations while allowing stale entries to grow without bound.

No new external crate is warranted for this correction unless the standard library/current dependencies cannot implement the bounded index cleanly; if a new dependency appears necessary, stop and create dependency-containment planning first.

### 5.4 Make peer capacity coherent with enabled rate windows

Compute the required tracked-peer budget from:

- the longest enabled per-peer rate window that requires historical state;
- the strongest enabled aggregate arrival bound that applies throughout that retention period;
- the configured global concurrency margin.

Under the default policy, the aggregate 50/minute ceiling applies continuously while per-peer day accounting is enabled, so capacity must not be an arbitrary 4096 entries that can be exhausted well before default day state begins expiring.

Required behavior:

1. define a hard maximum tracked-peer memory budget;
2. derive the peer-entry requirement implied by configured rates and retention;
3. include a small bounded concurrency/edge margin;
4. reject an unsafe configuration before session/task allocation if exact configured rate semantics would require state beyond the hard budget;
5. document worst-case bytes/entry and total admission-state memory at the hard maximum.

Do not silently weaken configured per-peer day/hour/minute limits by prematurely evicting non-expired records. Do not silently reinterpret an unlimited aggregate rate as a finite one.

If exact semantics plus an explicitly unlimited aggregate policy cannot be represented inside the selected memory ceiling, fail configuration truthfully and name the conflicting rate/capacity policy rather than accepting a runtime that predictably collapses into table exhaustion.

### 5.5 Retention must equal actual enabled semantics

Do not use a full-day retention merely because any per-peer rate is enabled. The record's required historical lifetime is the longest enabled per-peer window:

- minute only -> minute-scale retention;
- hour enabled -> hour-scale retention;
- day enabled -> day-scale retention;
- all peer rates unlimited -> only the short inactivity/concurrency retention necessary for active-state cleanup.

Window/accounting behavior itself remains exact and deterministic under Tokio paused time.

### 5.6 Preserve task-group/admission capacity equivalence

Retain one global configured ceiling shared by `ServerAdmissionPolicy` and `BoundedTaskGroup`.

Add an invariant test proving that a successfully acquired `AdmissionLease` is never silently lost because the task group rejects the corresponding spawn under ordinary single-runtime operation. If `try_spawn` can still fail after admission due to a real lifecycle race, explicitly drop/rollback the lease and accepted-stream work and cover that path. Do not ignore an unexpected capacity disagreement without evidence.

### 5.7 Dependency/test-util containment check

M074 enabled Tokio `test-util` on the normal `emissary-cli` Tokio dependency to support paused-time tests. Re-check whether this feature can be confined to test/dev dependency activation without changing production capability.

If it can, restore production dependency minimality in this corrective pass. If Cargo feature unification or existing workspace conventions make test-only confinement impossible without broader churn, record the exact reason in closure and do not create a dependency refactor.

This is a containment hygiene item only; it must not expand M080 into workspace dependency cleanup.

## 6. Failure, cancellation, restart, and contention semantics

- admission state remains per tunnel generation and ephemeral;
- restart creates a new empty admission state after old child tasks are drained/aborted;
- a lease from an old generation cannot mutate a new generation;
- panic/abort/EOF/handler error release global and peer active counts exactly once;
- denial under aggregate/peer/global/table pressure does not create persistent state;
- allocator reservation failure denies that connection cleanly and leaves logical state unchanged;
- expiry-index update failure must fail closed without orphaning a peer record;
- no mutex poisoning semantics are introduced (`parking_lot::Mutex` remains acceptable);
- contention work under the admission lock is bounded CPU/memory bookkeeping only.

## 7. Ordered work packages

### WP1 — Realistic peer identity fixture and keying

Introduce/reuse canonical Destination parsing and a fixed 32-byte peer key. Replace placeholder peer fixtures where needed.

### WP2 — Transactional admission

Reorder/check admission so all denial paths are mutation-free. Add internal test introspection for peer/counter/expiry sizes.

### WP3 — Bounded expiry index

Replace append-only stale expiry entries with one bounded authoritative deadline per tracked peer.

### WP4 — Capacity derivation

Implement retention-aware peer-capacity calculation and pre-allocation rejection for configurations whose exact semantics exceed the hard memory ceiling. Document selected hard ceiling and measured/derived worst-case memory.

### WP5 — Lifecycle/containment regression

Prove lease cleanup, task-group equivalence, restart reset, and evaluate the Tokio `test-util` dependency placement.

### WP6 — Documentation and closure repair

Update server admission documentation and create a new M080 closure. M074's historical implementation evidence may remain referenced, but M074 cannot be treated as current-security closed until M080 closes.

## 8. Required regression tests

At minimum:

- exhaust aggregate minute quota, then attempt more than the peer-state hard-cap count of fresh valid Destinations; rejected attempts do not increase peer records or expiry-index size;
- a new peer denied by aggregate rate leaves state exactly unchanged except reaping already-expired entries;
- a new peer denied by global concurrency leaves no peer record;
- an existing peer denied by peer rate does not increment any rate counter or extend expiry;
- an existing peer denied by aggregate rate does not increment peer counters or extend expiry;
- expiry-index live entries never exceed the peer/accounting capacity under repeated acquire/drop/refresh cycles;
- no stale expiry growth after thousands of successful acquire/drop cycles for one peer;
- canonical I2P Destination IDs produce stable distinct 32-byte keys for distinct fixtures;
- malformed Destination text is rejected before admission insertion/handler/local connect;
- default policy's derived capacity can represent the maximum aggregate-admissible distinct identities over its longest enabled peer window, or configuration fails before allocation with a documented bounded alternative;
- minute-only policy does not retain peer state for a day;
- explicit configuration whose exact state requirement exceeds the hard memory ceiling fails before SAM session allocation;
- all-unlimited peer rates use only bounded short retention/concurrency state;
- admission lease and task-group capacity do not diverge in a saturated accepted-server fixture;
- normal return, panic, cancellation, abort, and stop all release active counts;
- restarted generation begins with empty rate/peer state;
- error/debug output contains no full Destination or private key material.

## 9. Verification

Run at minimum:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol admission
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

Use Tokio paused-time tests for all retention/window evidence. Use the repository's scoped nightly rustfmt convention for touched Rust files only where required.

## 10. Acceptance criteria

M080 may close only when:

1. every admission denial path is proven mutation-free except bounded expiry reaping;
2. aggregate-rate rejection cannot create or retain a new peer record;
3. peer and expiry/index accounting are both hard bounded;
4. expiry bookkeeping has at most O(peer capacity) live state with no attacker-driven stale-entry accumulation;
5. peer identity accounting uses canonical cryptographic I2P Destination identity rather than `DefaultHasher`/64-bit keys;
6. configured/default rate windows and peer capacity are coherent, or impossible configurations reject before allocation;
7. retention is no longer unconditionally day-scale when only shorter peer windows are enabled;
8. global/per-peer fairness and Proposal 170 minute/hour/day semantics remain intact;
9. lease/task cleanup remains exact across error/panic/cancel/restart;
10. production changes remain I2PControl-contained and M061/M062/M063 pass;
11. the Tokio test-only feature is production-minimized where practical, or the closure records why not;
12. no high/medium admission/resource/correlation finding remains in M080 scope.

## 11. Stop conditions

Stop and create separate architecture/corrective planning if:

- canonical peer identity cannot be obtained from existing accepted-stream data without a new core/SAM API;
- exact configured rate semantics cannot be bounded without inventing a new public Proposal 170 control;
- representing safe default capacity requires an unacceptably large memory budget with no bounded configuration-time alternative;
- a new dependency or `emissary-core/**` production change appears necessary;
- fixing admission would require changing router streaming algorithms rather than I2PControl-owned bookkeeping.

External I2P/I2P+ source access remains read-only. M080 authorizes no upstream issue, PR, review, merge, contribution preparation, or repository write outside `eggstack/emissary`.
