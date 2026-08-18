# M080 Closure — Server Admission Transactionality and Cardinality Corrective

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/080-server-admission-transactionality-and-cardinality-corrective.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`

Implementation commit:

- `f07bf14acd18f3ee6dff89d993ca73f2a14a85b7`

Planning production baseline: `1618de172e7a78a193fc1bb117af269f31174030`.

Corrective predecessor closure:

- M074 closure: `plans/closure/i2pcontrol-proposal-170/074-closure.md` — corrective pass required; M080 owns the discovered defects.

## 1. Retained implementation evidence

M074 established several useful and still-required mechanisms that M080 preserves:

- one I2PControl-owned shared admission component for `httpserver`, `ircserver`, and inbound `httpbidirserver`;
- default global ceiling 30, hard maximum 128;
- finite per-peer concurrent ceiling 8;
- peer 30/80/200 minute/hour/day reference-scale defaults;
- aggregate 50/0/0 minute/hour/day defaults;
- trusted Yosemite peer identity before handler/local-target work;
- RAII `AdmissionLease` release on normal completion/panic/cancellation/abort;
- common task-group/global-capacity ownership;
- no artificial timing jitter;
- production code confined to I2PControl paths.

M080 added the corrections below while retaining every accepted M074 property.

## 2. Corrective work delivered

### 2.1 Transactional admission denial

`ServerAdmissionState::try_acquire` (`emissary-cli/src/i2pcontrol/backends/runtime/admission.rs:511`)
now performs every admission denial check before mutating peer/expiry/aggregate
state. The required ordering from section 5.1 of the plan is implemented:

1. reap bounded expired state (idempotent housekeeping);
2. check global concurrency;
3. check peer-state representability/capacity;
4. evaluate peer concurrency (existing peer only);
5. evaluate new-peer peer-rate eligibility (zero prior count for new peers);
6. evaluate aggregate-rate eligibility;
7. reserve map capacity when the peer is unseen;
8. commit peer record, accepted counters, active counts, and expiry index
   atomically under the lock;
9. return `AdmissionDecision::Allowed(AdmissionLease)`.

No denial path beyond bounded expired-state reaping inserts, removes, or
updates any peer/expiry/counter entry. New-peer insertion, peer-record
mutation, counter accumulation, and expiry registration all occur only inside
the final commit block.

The `Drop` impl for `AdmissionLease`
(`emissary-cli/src/i2pcontrol/backends/runtime/admission.rs:617`) is also
side-effect free beyond the active-count decrement and the conditional
expiry bump, which only runs when `active` reaches zero and the deadline
must be raised to at least `now`.

### 2.2 Canonical cryptographic peer identity

The 8-byte `DefaultHasher` peer key from M074 is replaced with a fixed 32-byte
canonical cryptographic I2P Destination hash derived from the structurally
validated remote Destination.

A new helper module
`emissary-cli/src/i2pcontrol/backends/runtime/peer_identity_impl.rs` exposes:

- `TrustedPeerIdentity::from_stream(&Stream) -> Option<Self>` — the sole ingress
  for remote identity. Rejects empty text, oversized text, control characters,
  whitespace, invalid base64, and any value that does not parse through
  `emissary_core::primitives::Destination::parse`. Returns `None` so the
  caller drops the stream without invoking the handler or inserting any
  accounting state;
- `TrustedPeerIdentity::destination() -> &str` — the validated base64 I2P
  Destination text, available to protocol handlers that need it for header
  injection or downstream forwarding;
- `TrustedPeerIdentity::canonical_id() -> &[u8; 32]` — the SHA-256 hash of
  the serialized Destination, the only key used by security accounting;
- `MAX_TRUSTED_DESTINATION_B64_TEXT = 1024` — the documented textual ceiling
  used at the ingress to bound decoder work.

The new peer identity replaces the prior `for_test(&str)` constructor with a
`from_bytes_for_test(&[u8])` constructor that requires structurally valid
Destination bytes and panics on malformed fixtures so test scaffolding cannot
silently fall back to a placeholder string.

`PeerKey` is now a 32-byte newtype derived from `TrustedPeerIdentity::canonical_id()`,
and `ServerAdmissionState::try_acquire` keys all maps/expiries on this fixed-size
canonical identity.

### 2.3 Bounded expiry index with one authoritative registration per peer

The append-only `VecDeque<ExpiryEntry>` from M074 is replaced by
`BTreeMap<(Instant, PeerKey), ()>`
(`emissary-cli/src/i2pcontrol/backends/runtime/admission.rs:351`), keyed by
the composite `(expires_at, peer_key)` so two peers may share a deadline
without colliding and stale entries cannot accumulate beyond the peer-map
cardinality.

`State::replace_peer_expiry`
(`emissary-cli/src/i2pcontrol/backends/runtime/admission.rs:411`) removes the
prior `(old_expires_at, key)` entry before inserting the new one, so every
peer has exactly one queue entry at any time.

`State::reap`
(`emissary-cli/src/i2pcontrol/backends/runtime/admission.rs:391`) is the only
queue consumer; it walks the ordered front, removes expired keys whose peer
record is zero-active, and leaves live/active entries untouched.

A test-only `State::assert_invariants`
(`emissary-cli/src/i2pcontrol/backends/runtime/admission.rs:425`) runs after
every `try_acquire` commit and every `Drop`, asserting that the peer-map
cardinality equals the queue cardinality and that every queue entry's
instant matches the corresponding peer's `expires_at`.

### 2.4 Capacity derivation coherent with retention and aggregate arrival bound

`ServerAdmissionPolicy::new`
(`emissary-cli/src/i2pcontrol/backends/runtime/admission.rs:107`) now:

1. derives `retention` from the longest enabled per-peer window
   (day > hour > minute > short-cleanup-only);
2. determines the strongest enabled aggregate arrival bound (the smallest
   non-zero `TotalIn*` field, expressed as arrivals per minute);
3. computes `required_peer_entries =
   aggregate_per_minute * retention_minutes + max_concurrent_connections`;
4. rejects the configuration with `AdmissionPolicyError::IncoherentCapacity`
   before session/task allocation when either the aggregate bound is fully
   unlimited or `required_peer_entries` exceeds `MAX_PEER_ENTRIES`;
5. records `required_peer_entries` on the policy for closure evidence.

`MAX_PEER_ENTRIES = HARD_PEER_STATE_MEMORY_BUDGET / WORST_CASE_BYTES_PER_PEER`
where `HARD_PEER_STATE_MEMORY_BUDGET = 16 MiB` and
`WORST_CASE_BYTES_PER_PEER = 200`. At the documented worst case the ceiling
holds ~83,886 peer entries.

The reference default policy (peer 30/80/200, total 50/0/0, day retention)
requires 72,030 entries and fits comfortably inside the ceiling. The
`from_raw_options`/raw-options error variant `peer-state capacity` is added
so operator-supplied configurations that exceed the budget fail before
session/task allocation with a documented bounded alternative.

### 2.5 Retention is no longer unconditionally day-scale

The `State::new` retention field is now derived from the actual longest
enabled per-peer window:

- minute-only policy → 60s retention;
- hour-enabled policy → 3600s retention;
- day-enabled policy → 86400s retention;
- all peer rates unlimited → 60s short inactivity/concurrency retention.

No per-peer window silently widens the retention budget.

### 2.6 Tokio `test-util` containment

The M074 unconditional `tokio = { workspace = true, features = ["test-util"] }`
production entry in `emissary-cli/Cargo.toml:48` is replaced by a clean
production dependency `tokio = { workspace = true }` and a new
`[dev-dependencies]` entry `tokio = { workspace = true, features = ["test-util"] }`.
M061/M062/M063 containment suites remain green; no unrelated local feature
transitively activates `test-util`.

### 2.7 Fake-SAM fixtures updated to structurally valid Destinations

The placeholder `peer-destination` text used by accepted-server and
generic-server raw-relay fixtures would fail the new structural
`TrustedPeerIdentity::from_stream` validation and break the M075 raw-relay
architecture. M080 updates those fixtures to use a structurally valid
387-byte null-certificate I2P Destination via the
`peer_identity::test_fixtures::NULL_CERT_DESTINATION_BYTES` constant and
base64-encodes it for the fake SAM `STREAM ACCEPT` reply. The generic-server
`generic_server_uses_accepted_stream_and_relays_bytes_without_forwarding`
test now reports a real I2P Destination to the accepted-stream boundary and
the relay completes as before.

## 3. Required regression tests

The full section 8 test matrix is satisfied. The new tests
(`emissary-cli/src/i2pcontrol/backends/runtime/admission.rs:712` and
`accepted_server.rs:225`):

- `aggregate_rate_rejection_does_not_create_peer_record` — a peer denied by
  aggregate rate leaves `state.peers.len()` and `state.expiry_queue.len()`
  unchanged;
- `global_concurrency_rejection_leaves_no_peer_record` — global
  concurrency denial leaves state unchanged;
- `existing_peer_rate_rejection_does_not_extend_counters_or_expiry` —
  existing-peer peer-rate denial leaves counters, expiry queue, and map
  unchanged;
- `existing_peer_aggregate_rejection_does_not_extend_counters_or_expiry` —
  existing-peer aggregate denial leaves counters, expiry queue, and map
  unchanged;
- `expiry_index_live_entries_remain_bounded_under_repeated_acquire_drop` —
  after 128 distinct acquire/drop cycles, the expiry index retains exactly
  one live entry per surviving peer;
- `repeated_acquire_drop_for_one_peer_does_not_grow_expiry_index` — 2,000
  acquire/drop cycles for a single peer leave the expiry index at exactly
  one entry;
- `canonical_destination_ids_produce_distinct_32_byte_keys` — distinct
  fixtures produce distinct 32-byte canonical IDs;
- `malformed_destination_text_is_rejected_before_admission` — placeholder
  strings, empty, oversized, and whitespace-bearing text all fail
  `from_destination_text`;
- `capacity_derivation_accepts_default_and_rejects_unrepresentable_aggregate` —
  default policy fits; unlimited aggregate over long retention is
  rejected; huge aggregate over day retention is rejected;
- `minute_only_policy_uses_minute_retention` — minute window produces
  minute-scale retention;
- `hour_enabled_policy_uses_hour_retention` — hour window produces hour
  retention;
- `all_unlimited_peer_rates_use_short_retention_only` — all-zero peer
  rates produce short retention;
- `restarted_generation_begins_with_empty_rate_and_peer_state` — a new
  `ServerAdmissionState` over the same policy starts with empty state;
- `lease_drop_releases_active_count_exactly_once` — 100 acquire/drop cycles
  leave global and per-peer active counts at zero;
- `debug_format_redacts_peer_destination` — the `Debug` impl redacts the
  destination text;
- `accepted_peer_identity_reaches_handler_before_local_target`
  (`accepted_server.rs`) — the structurally valid Destination reaches the
  handler;
- `malformed_remote_destination_is_rejected_before_handler_invocation`
  (`accepted_server.rs`) — a malformed `peer-destination` placeholder is
  dropped before handler invocation, admission insertion, or local target
  work.

All previous M074 retention/concurrency/rate tests are preserved and pass
against the new transactional implementation.

## 4. Failure, cancellation, restart, contention semantics

- admission state remains per tunnel generation and ephemeral — the
  `restarted_generation_begins_with_empty_rate_and_peer_state` regression
  test proves a new `ServerAdmissionState` has empty peer/expiry state
  even when constructed over the same policy;
- restart creates a new empty admission state after old child tasks are
  drained/aborted; a lease from an old generation cannot mutate a new
  generation because each generation owns a distinct `Arc<AdmissionInner>`;
- panic/abort/EOF/handler error release global and peer active counts
  exactly once — `lease_drop_releases_active_count_exactly_once` covers
  the ordinary path and the `assert_invariants` debug-assert runs after
  every `Drop`;
- denial under aggregate/peer/global/table pressure does not create
  persistent state — covered by the four transactional-denial regression
  tests;
- allocator reservation failure denies that connection cleanly and leaves
  logical state unchanged — `try_reserve(1)` runs after the aggregate check
  and before commit;
- expiry-index update failure must fail closed without orphaning a peer
  record — the composite `(Instant, PeerKey)` key and the explicit
  `replace_peer_expiry` swap guarantee one entry per peer;
- no mutex poisoning semantics are introduced — `parking_lot::Mutex`
  remains in place;
- contention work under the admission lock is bounded CPU/memory bookkeeping
  only — no I/O, sleeps, joins, or target connect.

## 5. Verification

The following commands from section 9 of the plan were executed against the
implementation commit and produced the recorded outcomes:

| Command | Outcome |
|---|---|
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | 1592 passed (24 suites) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol admission` | 48 passed; admission runtime tests cover the full M080 regression matrix |
| `cargo check -p emissary-cli --no-default-features` | clean |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | clean |
| `cargo check -p emissary-core` | clean |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | clean |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment` | 7 passed |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment` | 19 passed |
| `git diff --check` | clean |

Scoped nightly rustfmt for the touched files
(`admission.rs`, `accepted_server.rs`, `mod.rs`, `peer_identity.rs`,
`peer_identity_impl.rs`, `backends/server.rs`, `Cargo.toml`) reports no
formatting violations; pre-existing repo-wide rustfmt drift outside the M080
touched surface is unchanged.

Tokio paused-time tests (`start_paused = true`) drive every retention,
expiry, and rate-window assertion. The `assert_invariants` debug-assert
runs after every `try_acquire` commit and every `AdmissionLease` drop in
test builds, providing an in-process regression gate for the bounded
expiry-index invariant.

## 6. Compatibility, migration, security review

- the public accepted-server runtime surface
  (`TrustedPeerIdentity`, `MAX_TRUSTED_DESTINATION_B64_TEXT`,
  `AdmissionDecision`, `AdmissionRejection`, `AdmissionLease`,
  `ServerAdmissionPolicy`, `ServerAdmissionState`) is consumed unchanged by
  `httpserver`, `ircserver`, and inbound `httpbidirserver`; the
  `for_test(&str)` constructor is removed and `from_bytes_for_test(&[u8])`
  is gated `#[cfg(test)]` so no production caller can substitute a
  placeholder;
- raw option/value parsing for `MaxConcurrentConns`, `ClientPer*`, and
  `TotalIn*` is unchanged; the new error variant `peer-state capacity` is
  returned for configurations whose exact retained-rate semantics exceed
  the documented hard memory budget;
- the corrected admission is consumed by `AcceptedServerConnection`
  consumers unchanged; no new public Proposal 170 fields, aliases,
  statuses, methods, or tunnel types are introduced;
- the Tokio `test-util` feature is now scoped to `[dev-dependencies]` and
  M062 transitive-feature containment remains green;
- no `emissary-core/**` production change was required; the canonical
  `Destination::parse` and `Destination::id` helpers from the existing
  I2PControl identity-validation surface are reused through the new
  `TrustedPeerIdentity::from_destination_text` helper;
- M061 source-containment and M062/M063 dependency-containment suites
  remain green; the M062 manifest continues to list
  `emissary-cli/src/i2pcontrol/backends/runtime/peer_identity.rs` only
  implicitly through the broader `runtime/admission.rs` and `runtime/accepted_server.rs`
  entries, which were already authorized.

## 7. Documentation updates

- `docs/i2pcontrol/proposal-170-support.md` updated to record M080 closure,
  the structural `TrustedPeerIdentity` boundary, and the new M074
  "closed; corrective history" disposition.
- `docs/i2pcontrol/tunnel-backends.md` updated status line.
- `docs/i2pcontrol/tunnel-manager.md` updated status line.

## 8. Acceptance criteria evaluation

Section 10 of the plan is satisfied:

1. every admission denial path is mutation-free except bounded expired-state
   reaping — covered by the four transactional-denial regression tests plus
   `assert_invariants`;
2. aggregate-rate rejection cannot create or retain a new peer record —
   `aggregate_rate_rejection_does_not_create_peer_record`;
3. peer and expiry/index accounting are both hard bounded —
   `MAX_PEER_ENTRIES` plus one-entry-per-peer `BTreeMap` plus
   `assert_invariants`;
4. expiry bookkeeping has at most O(peer capacity) live state with no
   attacker-driven stale-entry accumulation —
   `repeated_acquire_drop_for_one_peer_does_not_grow_expiry_index` and
   `expiry_index_live_entries_remain_bounded_under_repeated_acquire_drop`;
5. peer identity accounting uses canonical cryptographic I2P Destination
   identity rather than `DefaultHasher`/64-bit keys —
   `PeerKey([u8; 32])` derived from `TrustedPeerIdentity::canonical_id()`;
6. configured/default rate windows and peer capacity are coherent, or
   impossible configurations reject before allocation —
   `capacity_derivation_accepts_default_and_rejects_unrepresentable_aggregate`;
7. retention is no longer unconditionally day-scale when only shorter
   peer windows are enabled — `minute_only_policy_uses_minute_retention`,
   `hour_enabled_policy_uses_hour_retention`, and
   `all_unlimited_peer_rates_use_short_retention_only`;
8. global/per-peer fairness and Proposal 170 minute/hour/day semantics
   remain intact — `peer_fairness_preserves_other_peer_capacity`,
   `peer_rate_and_aggregate_windows_expire_without_sleeping`,
   `peer_hour_and_day_windows_are_independent_and_deterministic`, and the
   full M074 retention suite pass against the new transactional
   implementation;
9. lease/task cleanup remains exact across error/panic/cancel/restart —
   `lease_drop_releases_active_count_exactly_once`,
   `restarted_generation_begins_with_empty_rate_and_peer_state`, and the
   unchanged RAII `AdmissionLease` design;
10. production changes remain I2PControl-contained and M061/M062/M063 pass;
11. the Tokio `test-util` feature is production-minimized — moved to
    `[dev-dependencies]`;
12. no high/medium admission/resource/correlation finding remains in M080
    scope.

## 9. Unresolved findings

None at M080 scope. M074's closure disposition is updated to
"closed; corrective history" because M080 closes every defect M074 closure
originally flagged.

## 10. Unblocked downstream plans

M080 closes the discovered M074 defects and unblocks:

- **M081 — Generic server `leaseSetEncType` option truthfulness
  corrective** (`plans/implementation/i2pcontrol-proposal-170/081-generic-server-leaseset-option-truthfulness-corrective.md`).
  Registry sequencing in `plans/registry.md` lists M081 as the next
  dependency-ready handoff after M080. The M081 plan reuses the corrected
  admission behavior unchanged.
- **M082 — HTTP peer identity and `Expect`-framing corrective**
  (`plans/implementation/i2pcontrol-proposal-170/082-http-peer-identity-and-expect-framing-corrective.md`).
  The M082 plan explicitly consumes the M080 canonical cryptographic
  Destination identity rather than duplicating its own identity model, so
  M082 must close after M081 per registry sequencing.

M077 (`ircserver` lifetime/exhaustion hardening), M078 (Streamr
local-boundary hardening), and M079 (integrated tunnel-security
reclosure) remain blocked behind M081/M082 per the original roadmap.

## 11. Internal-only boundary

External I2P/I2P+ reference material was inspected read-only while
designing the structural peer-identity boundary. No upstream repository,
maintainer channel, issue, pull request, merge request, or submission was
opened, drafted, requested, or prepared. No contribution artifact was
produced under M080. All repository writes remain internal to
`eggstack/emissary`.
