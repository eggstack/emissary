# M083 — Admission Capacity Semantics and Trusted Destination Exactness Corrective

Status: closed — implementation commit `3eaea53`; closure accepted in
`plans/closure/i2pcontrol-proposal-170/083-closure.md`; M077 is now ready

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`.

Corrective predecessors:

- M080 — `080-server-admission-transactionality-and-cardinality-corrective.md`;
- M082 — `082-http-peer-identity-and-expect-framing-corrective.md`.

Planning production baseline: `a35d2bc333ff0e8b9889cd133d8ef75a98faa049`.

## 1. Objective

Close the remaining post-M082 defects in the shared accepted-server security boundary without redesigning that boundary.

M083 must:

1. make peer-state capacity representability correct for minute-scale and no-history policies, including explicitly unlimited aggregate rates;
2. compute capacity from the tightest safe upper bound implied by **all** enabled aggregate windows rather than field precedence;
3. preserve a coherent bounded expiry-index invariant when a peer remains active beyond its nominal expiry;
4. require a trusted peer identity to decode to exactly one supported I2P Destination and expose only canonical Destination text downstream.

The M080 transactional-denial architecture, 32-byte Destination ID accounting, bounded task ownership, M081 `leaseSetEncType` correction, and M082 HTTP `Expect`/POST-key corrections are retained.

## 2. Findings requiring this corrective

### 2.1 MEDIUM — minute peer history bypasses capacity representability

`ServerAdmissionPolicy::new` currently derives both:

- `MINUTE = 60s` for an enabled `ClientPerMinute`; and
- `SHORT_RETENTION = 60s` when all per-peer rates are unlimited.

Capacity validation runs only when:

```text
retention > SHORT_RETENTION
```

A minute-only peer policy therefore takes the same branch as the no-history cleanup policy. For example:

```text
ClientPerMinute > 0
ClientPerHour = 0
ClientPerDay = 0
TotalInPerMinute = 0
TotalInPerHour = 0
TotalInPerDay = 0
```

is accepted even though aggregate arrivals are explicitly unlimited and distinct peer records must remain for the minute peer-rate window. Fresh authenticated Destinations can fill the bounded peer table before that history expires and force `PeerStateCapacity` for legitimate new peers.

This contradicts M080's requirement that exact retained-rate semantics with an unrepresentable unlimited aggregate policy fail before session/task allocation.

### 2.2 MEDIUM — no-history inactive records can still create avoidable churn pressure

When all per-peer rates are unlimited, no historical peer counter state is required after the final active connection for that peer closes. The current implementation nevertheless retains inactive peer records for `SHORT_RETENTION`.

With an unlimited aggregate policy, sequential fresh identities can therefore consume the peer table for an arbitrary cleanup interval even though only active concurrency state is semantically required.

M083 must separate **historical peer-rate retention** from **active connection ownership**. When no peer-rate history is enabled, an inactive peer must not remain solely because of an arbitrary cleanup duration. Active records remain bounded by the existing global/per-peer concurrency limits.

### 2.3 MEDIUM-LOW — aggregate capacity solver does not select the tightest enabled bound

`strongest_aggregate_per_minute` currently uses field precedence:

1. `TotalInPerMinute` when non-zero;
2. otherwise `TotalInPerHour` converted to per-minute;
3. otherwise `TotalInPerDay` converted to per-minute.

The aggregate limiter enforces all enabled fields conjunctively. A configuration such as a permissive minute limit plus a much tighter hour/day limit can therefore be rejected as `IncoherentCapacity` even though its actual maximum retained peer cardinality fits the hard memory budget.

M083 must derive capacity from the intersection of enabled limits, not from the first non-zero field.

### 2.4 MEDIUM-LOW — fixed-window boundary overlap must be included in the capacity proof

Capacity is a bound on the maximum distinct accepted identities that can remain live during a peer-history interval. Aggregate counters are fixed monotonic windows. A retained interval may straddle a window boundary and observe accepted traffic from both adjacent fixed windows.

A naive `rate * retention/window` calculation can therefore understate the number of accepted identities that may coexist.

M083 must use a conservative bound that cannot underestimate the actual fixed-window implementation. For each enabled aggregate window `(limit, window)`, compute a safe maximum accepted-event count over the peer-history horizon including partial-window overlap. A suitable conservative form is:

```text
limit * (ceil(history_window / aggregate_window) + 1)
```

or a tighter equivalent proven against the actual counter semantics. Take the minimum safe bound across all enabled aggregate fields, because every enabled aggregate limiter applies. Add the documented global-concurrency/edge margin after selecting the tightest safe bound.

Do not use an approximation that can under-budget attacker-controlled state.

### 2.5 LOW-MEDIUM — reaping an expired active peer can remove its authoritative expiry entry

`State::reap` collects expired peer keys, then may remove the peer's `(expires_at, key)` expiry entry even when the peer is still active and therefore not removable from `peers`.

That transiently violates M080's claimed peer-map/expiry-index relationship. A later drop may repair the entry, but the state machine must not rely on a future event to restore its own structural invariant.

M083 must define and enforce one of these bounded invariants:

- every tracked peer has exactly one authoritative expiry-index entry, including active peers; or
- the expiry index contains exactly one authoritative entry for every **inactive** peer eligible for time-based reaping, while active peers are intentionally unindexed and bounded by concurrency.

Either design is acceptable if it is explicit, deterministic, bounded, and covered across acquire/reap/drop transitions. Accidental orphaning is not acceptable.

### 2.6 LOW-MEDIUM — trusted Destination parsing accepts unconsumed trailing bytes

`TrustedPeerIdentity::from_destination_text` Base64-decodes and calls `Destination::parse`. The core convenience parser returns the parsed Destination while discarding `parse_frame`'s unconsumed remainder.

Consequently, input containing one valid Destination followed by trailing bytes can be treated as trusted. Security accounting remains keyed on the parsed 32-byte Destination ID, but the original textual value is retained and can reach HTTP identity headers and exact textual access-list comparisons.

M083 must require the decoded payload to contain exactly one supported Destination and no unconsumed bytes. The correction stays I2PControl-local; do not change the core parser.

Downstream full-Destination text must be canonical text derived from the validated serialized Destination, not an attacker-selected textual alias. Canonical ID derivation remains unchanged.

## 3. Scope and containment

Preferred production changes:

```text
emissary-cli/src/i2pcontrol/backends/runtime/admission.rs
emissary-cli/src/i2pcontrol/backends/runtime/peer_identity_impl.rs
```

Allowed only when directly required for focused regressions/integration:

```text
emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs
emissary-cli/src/i2pcontrol/backends/filters/http.rs
emissary-cli/src/i2pcontrol/backends/http_server.rs
```

Planning/docs/containment metadata may be updated as required for closure.

Hard exclusions:

- no `emissary-core/**` production change;
- no router, streaming, SAM, or Yosemite protocol extension;
- no new tunnel type or Proposal 170 wire field;
- no new third-party dependency;
- no generalized rate-limiter/cache framework;
- no HTTP feature expansion;
- no IRC/Streamr implementation work inside M083;
- no hosted CI, generalized fuzzing, soak farm, release machinery, or benchmark gate;
- no upstream issue, PR, review, merge, submission, contribution preparation, or maintainer contact.

If exact Destination consumption or capacity correctness requires a core/protocol change, stop and create separate architecture planning rather than widening M083.

## 4. Capacity-model requirements

### 4.1 Separate peer-history semantics from cleanup mechanics

Do not infer whether historical peer state is required from `Duration` equality/ordering.

Represent it explicitly, for example with an `Option<Duration>`, enum, or equivalent internal state:

```text
None       -> no peer minute/hour/day history required
Minute     -> 60-second peer history
Hour       -> 1-hour peer history
Day        -> 24-hour peer history
```

The exact internal representation is implementation-owned. The semantic distinction is mandatory.

### 4.2 No-history policy

When all `ClientPerMinute`, `ClientPerHour`, and `ClientPerDay` values are zero:

- active peer state remains for concurrency accounting;
- once the peer's final lease drops, no peer-rate history is required;
- inactive peer state must be removed promptly rather than retained for an arbitrary 60-second cleanup window;
- unlimited aggregate rates are therefore representable because attacker-controlled inactive cardinality does not accumulate beyond active concurrency.

Do not silently add a finite aggregate limit to make the configuration fit.

### 4.3 Historical peer-rate policy

When any peer-rate window is enabled:

1. derive the longest enabled peer-history horizon;
2. compute a conservative maximum accepted-event count over that horizon for every enabled aggregate rate;
3. include fixed-window boundary overlap;
4. take the minimum of those safe bounds;
5. add the finite concurrency/edge margin using checked/saturating arithmetic;
6. reject `IncoherentCapacity` before allocation when no aggregate bound is enabled or when the required peer entries exceed `MAX_PEER_ENTRIES`.

No non-expired peer-rate state may be evicted early to make room.

### 4.4 Preserve configured rate semantics

M083 is capacity bookkeeping, not a policy rewrite. Preserve:

- `0` means unlimited for individual Proposal 170 rate fields;
- minute/hour/day counters remain independently enforced;
- M074/M080 reference defaults remain 30 global, 8 per peer, peer 30/80/200, aggregate 50/0/0;
- operator values within existing numeric bounds retain their requested meaning;
- impossible combinations fail before destination/session/task allocation.

## 5. Expiry-index requirements

The corrected state machine must remain O(bounded peer state) and must not create stale expiry metadata.

Required transitions:

```text
new accepted peer
existing accepted peer
peer/global/aggregate denial
lease drop while other leases remain
final lease drop
reap while peer is inactive
reap while peer is active past nominal expiry
restart/new generation
```

For every transition, tests must assert the selected peer-map/expiry-index invariant.

`reap` should operate on the actual `(deadline, key)` entry rather than reconstructing/removing a different composite key from a key-only collection. No expired active peer may become accidentally unindexed.

## 6. Trusted Destination exactness requirements

Keep the existing 1024-character ingress work bound unless evidence requires a smaller bound that still covers every repository-supported Destination form.

Within that bound:

1. reject empty/control/whitespace-invalid input as today;
2. Base64-decode once;
3. parse with the existing core `Destination::parse_frame` or equivalent existing primitive;
4. require `rest.is_empty()` — one complete Destination and no trailing bytes;
5. reject unsupported certificate/key forms exactly as the existing parser does;
6. derive the 32-byte canonical ID from the parsed Destination;
7. derive stored/forwarded full-Destination text by canonical I2P Base64 encoding of the parsed serialized bytes.

Do not hash or account on the original textual representation.

Do not loosen the core parser or add new supported key/certificate types in M083.

## 7. HTTP compatibility regression

M083 does not reopen the M082 HTTP design. It must prove that canonicalized trusted identity continues to feed the existing HTTP path correctly:

- `X-I2P-DestB64` is the canonical full Destination text;
- `X-I2P-DestB32` is derived from the same canonical Destination;
- POST limiting remains keyed on `canonical_id()`;
- `Expect` still returns fixed 417 before local connect;
- proxy/I2P spoof stripping and response fingerprint filtering remain unchanged;
- canonical access-list entries match the authenticated peer;
- trailing-byte/non-exact identities fail before HTTP request construction or limiter state.

No new HTTP informational-response handling is authorized.

## 8. Ordered work packages

### WP1 — Explicit peer-history model

Separate rate-history retention from no-history active cleanup. Add direct policy tests before changing capacity math.

### WP2 — Safe aggregate-capacity solver

Replace field-precedence conversion with a conservative per-window cardinality bound, choose the tightest enabled bound, include boundary overlap, and fail unrepresentable historical policies before allocation.

### WP3 — Expiry state-machine repair

Repair active-peer reap semantics and encode one explicit bounded peer/index invariant across acquire/drop/reap.

### WP4 — Exact/canonical trusted Destination

Require zero parser remainder and canonicalize downstream full-Destination text using existing primitives.

### WP5 — Cross-boundary regression and closure reconciliation

Re-run accepted-server and HTTP regressions, containment tests, and update M080/M082 current dispositions. Only then advance M077.

## 9. Required tests

At minimum:

### Capacity/history

- minute-only peer history + all aggregate fields unlimited rejects `IncoherentCapacity` before allocation;
- hour-only peer history + all aggregate fields unlimited rejects;
- day peer history + all aggregate fields unlimited rejects;
- all peer-rate fields unlimited + all aggregate fields unlimited remains valid **and sequential fresh inactive peers do not accumulate** after lease drop;
- minute peer history + bounded minute aggregate derives a finite representable requirement;
- permissive minute aggregate + tighter hour aggregate uses the tighter safe hour-derived bound and does not false-reject a representable policy;
- permissive minute/hour aggregates + tighter day aggregate uses the day-derived bound;
- a tighter minute aggregate remains authoritative when it is actually the smallest safe bound;
- fixed-window boundary tests use paused time to accept traffic immediately before/after a reset and prove `required_peer_entries` is never below observed retained distinct identities;
- reference defaults remain representable inside the existing hard memory ceiling;
- configurations genuinely above the hard ceiling still fail before session/task allocation;
- no checked-arithmetic overflow can wrap capacity downward.

### Expiry state

- active peer surviving past nominal expiry followed by unrelated admission does not lose required expiry/index representation;
- final drop after an active-over-expiry interval leaves exactly the documented inactive/index state;
- repeated reap calls are idempotent;
- repeated acquire/drop/reap cannot grow expiry metadata beyond the documented bound;
- denial paths remain side-effect-free apart from bounded expired-state reclamation;
- restart/new generation begins with empty state.

### Trusted identity

- canonical 387-byte null-certificate Destination is accepted;
- canonical repository-supported key-certificate Destination is accepted;
- valid Destination bytes plus one trailing byte are rejected;
- valid Destination bytes plus arbitrary trailing payload are rejected;
- malformed/truncated Base64 remains rejected;
- canonical ID is unchanged for the same valid Destination;
- downstream `destination()` returns canonical re-encoded text, not attacker-selected raw text;
- Debug/errors continue to redact destination text and ID.

### HTTP regression

- canonical trusted identity reaches `X-I2P-DestB64`/B32 correctly;
- non-exact/trailing identity never reaches HTTP request construction, POST limiter, or local target;
- M082 `Expect` no-local-connect regression remains green;
- existing fingerprint/proxy/framing tests remain green.

## 10. Verification

Use proportional local verification only:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol admission
cargo test -p emissary-cli --no-default-features --features i2pcontrol http
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

Use Tokio paused time for capacity/window/reap tests. Run repository-accepted scoped nightly rustfmt on touched Rust files.

Do not add new CI jobs or generalized verification infrastructure.

## 11. Acceptance criteria

M083 may close only when all of the following are true:

1. minute/hour/day peer-history semantics are explicitly distinguished from no-history active cleanup;
2. no historical peer-rate policy with fully unlimited aggregate arrivals is accepted unless a finite representability proof exists;
3. no-history inactive peer churn can accumulate a table-sized denial state;
4. capacity uses the tightest safe bound across all enabled aggregate windows rather than field precedence;
5. fixed-window boundary overlap cannot make actual retained distinct identities exceed the calculated requirement;
6. the reference default policy remains representable inside the hard memory ceiling;
7. expiry-index state has one documented, tested invariant across active-expired/reap/drop transitions and remains hard bounded;
8. trusted peer text decodes to exactly one supported Destination with no trailing bytes;
9. downstream full-Destination text is canonicalized from parsed bytes and accounting remains keyed by the 32-byte Destination ID;
10. M081 `leaseSetEncType`, M082 `Expect`/POST-key, M076 HTTP anonymity, and M080 transactional-denial behavior remain green;
11. production changes remain within I2PControl; no new core production path or dependency appears;
12. M061/M062/M063 containment remains green;
13. no high/medium finding remains in M083 scope;
14. closure explicitly updates the current disposition of M080 and the inherited trusted-identity portion of M082;
15. no upstream interaction occurred.

Only after M083 closes may the registry advance M077 to `ready`.

## 12. Stop conditions

Stop and create separate planning if:

- exact Destination validation requires changing `emissary-core::primitives::Destination` rather than composing existing parse primitives locally;
- exact capacity semantics require weakening or silently rewriting Proposal 170 rate values;
- the hard memory ceiling must be raised materially without a bounded memory justification;
- fixing expiry ownership requires a generalized runtime/task framework outside I2PControl;
- a new dependency is proposed;
- implementation pressure expands into IRC, Streamr, router streaming, SAM protocol, or HTTP feature work.

External specifications/reference implementations remain read-only evidence. No upstream review, issue, pull request, merge, submission, or contribution preparation is authorized.
