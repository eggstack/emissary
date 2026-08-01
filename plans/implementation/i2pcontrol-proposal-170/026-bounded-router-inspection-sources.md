# M026 — Bounded Router Inspection Sources

Status: implemented

Primary class: infrastructure/capability corrective pass

Hard dependency:

- M025 closed for implementation with a frozen owner-grouped feasible-source matrix

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Prior defect record:

- `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md`

## 1. Bounded objective

Add the smallest read-only, bounded inspection seams needed for Proposal 170 RouterInfo selectors that M025 proves are both semantically exact and already represented by authoritative state inside Emissary.

This milestone does not promise to make every unavailable field available. It must implement feasible groups without changing router behavior and leave the remainder explicitly unavailable.

The implementation agent must treat the M025 matrix as the complete input. It must not discover and opportunistically add unrelated telemetry.

## 2. Preconditions

M025 must provide, for every candidate field:

- exact key and JSON type;
- authoritative owner module/type;
- proof that the underlying state already exists;
- bounded snapshot shape;
- maximum collection size/byte budget;
- freshness semantics;
- explicit reason the snapshot does not change owner behavior;
- focused fixture expectation.

Any candidate lacking one of these is not ready and remains unavailable.

M025's frozen input currently marks every unavailable field as `deferred
unavailable` or `out of scope`; there are no `M026 feasible` fields. The
owner-grouped audit input and exact reasons are recorded in section 13 of the
M025 plan and its implementation disposition. M026 must preserve that result
unless new authoritative state is demonstrated without expanding scope.

## 3. Required invariants

1. Every new source is read-only from the I2PControl perspective.
2. The existing subsystem remains the sole owner of mutation, task lifecycle, queues, routing, peer state, bans, and counters.
3. Snapshot size is bounded before exposure.
4. Snapshot acquisition does not consume a single-owner receiver or remove events from another subsystem.
5. Snapshot acquisition does not hold owner locks across await points.
6. No new polling, periodic sampler, database, history buffer, or background task is added solely for Proposal 170.
7. No field is approximated from a semantically different aggregate.
8. Failure is explicit; no empty/zero fabricated success.
9. Private keys, tunnel keys, destination secrets, payloads, and sensitive paths are excluded.
10. Existing router algorithms, timing, and network behavior are unchanged.
11. External changes remain adjacent to the authoritative owner and expose purpose-specific DTOs/handles only.
12. Fields that cannot meet these invariants remain unavailable and block unqualified full-support claims.

## 4. Explicit non-goals

- no missing tunnel data-plane implementation;
- no new tunnel/peer/transport algorithms;
- no Java-I2P/i2pd category emulation when Emissary lacks the category;
- no 15-second/recent rate sampler if no current source exists;
- no new ban system, connection limiter, or queue;
- no full RouterInfo export service beyond the exact bounded fields;
- no generic introspection framework or event bus;
- no frontend, CI, release, dependency, performance project, or upstream work.

## 5. Permitted owner groups

Execute only groups marked `M026 feasible` by M025. Expected groups may include the following, but this list is not authority without the frozen matrix.

### Group A — Existing event/network status

Potential fields:

- IPv4/IPv6 status/testing/error codes when exact mappings already exist;
- current connection limits from validated configuration;
- current tunnel-build queue sizes if directly owned and bounded;
- current build success counters/rates if existing counters match exact time semantics.

Do not derive specific error codes from generic firewall status unless the mapping is exact and documented.

### Group B — Tunnel pools and participating tunnels

Potential fields:

- participating tunnel details;
- exploratory inbound/outbound counts and bounded info lists;
- client inbound/outbound counts and bounded info lists;
- tunnel build message/request queue sizes.

Expose only sanitized identifiers and metrics required by the pinned shape. No tunnel keys, gateway secrets, or message contents.

Do not change pool ownership or lifecycle.

### Group C — NetDB and peer snapshots

Potential fields:

- known peer hashes;
- active peer hashes;
- bounded serialized public RouterInfo for known/active peers;
- current NTCP/SSU connection limits;
- active peer stats already tracked;
- banned peer details only if an authoritative ban owner exists.

Do not scan unbounded disk state per request. Use current bounded in-memory indexes/snapshots. Do not expose profiles, private addresses not required by the contract, or mutable handles.

### Group D — Existing recent metrics

Potential fields only when a matching rolling window already exists:

- 15-second transit bandwidth;
- recent tunnel success rate.

If Emissary does not already maintain the exact window, leave unavailable. Do not add a sampler in this milestone.

## 6. Expected file boundary

I2PControl adapter/DTO files:

- `emissary-cli/src/i2pcontrol/router_info.rs`
- `emissary-cli/src/i2pcontrol/production.rs`
- `emissary-cli/src/i2pcontrol/router_info_handler.rs`
- focused tests/docs.

Permitted core changes are only adjacent read-only snapshot methods/types in owner modules identified by M025, potentially under:

- event/network status owner;
- tunnel manager/pool/build-queue owner;
- NetDB owner;
- peer/transport manager owner;
- ban owner if one exists.

`emissary-cli/src/main.rs` may pass existing handles/configuration at composition. No other files are authorized without a stop-and-record decision.

## 7. Required work packages

### WP1 — Implement one owner group at a time

For each M025-feasible group:

1. define a small immutable snapshot DTO in or adjacent to the owner;
2. bound every collection and string/serialized byte field;
3. add one read-only handle method that clones/copies the snapshot;
4. map it through the existing I2PControl production adapter;
5. serialize exact Proposal 170 keys/types;
6. add focused owner and handler tests;
7. record fields made available and fields still unavailable before proceeding to the next group.

Do not combine owner groups in one large refactor.

### WP2 — Bounded public RouterInfo serialization

If peer RouterInfo lists are feasible:

- serialize only public RouterInfo bytes already owned by NetDB;
- enforce item and total-byte bounds;
- use deterministic peer ordering;
- avoid disk reads and network queries during request handling;
- fail explicitly on an oversized result rather than truncate;
- never expose local private identity or key material.

### WP3 — Tunnel detail sanitization

If tunnel detail lists are feasible:

- define the exact field map from the pinned proposal/reference;
- include only non-secret IDs, direction, state, age/expiry, peer hashes, or rates explicitly required;
- redact/omit tunnel IDs or peer details if the contract does not require them;
- bound tunnel counts and per-entry sizes;
- snapshot pool state without affecting scheduling.

### WP4 — Status and limit mapping

- Return current configured limits from the configuration owner when exact.
- Map network status/error/testing codes only through a documented exhaustive conversion.
- Unknown internal states must map to the exact protocol-defined unknown/error value if one exists; otherwise field remains unavailable.
- Add compile-time/exhaustiveness guards where enums are involved.

### WP5 — Query grouping and caching within one request

- Query each owner group once per request.
- Reuse one immutable snapshot for all requested selectors in that group.
- Do not add cross-request caching or stale background refresh.
- Release owner locks before serialization.

### WP6 — Unavailable-field preservation

For each rejected group/field:

- keep the M025 unavailable classification;
- add a test that no fabricated value appears;
- document the exact missing owner/state and why adding it would exceed scope;
- do not leave TODOs that imply silent completion.

## 8. Failure, cancellation, restart, and contention semantics

- Snapshot failure aborts the RouterInfo request without partial results.
- Concurrent owner mutation yields a coherent snapshot according to the owner's existing synchronization.
- Request cancellation drops the cloned snapshot with no owner effect.
- Restart reconstructs sources through normal owner startup; I2PControl stores no duplicate telemetry state.
- Oversized snapshots fail before response serialization completes and do not mutate owner state.
- No new contention point may be on a packet, tunnel-build, or peer-routing hot path beyond a bounded short read/copy already acceptable to the owner.

## 9. Security and performance constraints

- Benchmarking is not a closure gate, but code review must identify whether any snapshot traverses a hot-path collection.
- Prefer existing counters and immutable/public records.
- Do not clone unbounded maps.
- Apply conservative fixed bounds and actual serialized-byte checks.
- Sanitize errors to owner-group names; no peer contents, paths, or internal lock/debug state.
- No response field may expose a private key, tunnel key, session key, payload, authentication token, or operator credential.

## 10. Focused tests

For every implemented owner group:

1. Empty bounded snapshot exact output.
2. Representative populated exact output.
3. Deterministic ordering.
4. Count bound exceeded returns explicit error.
5. Byte bound exceeded returns explicit error.
6. Owner query failure propagates and produces no partial result.
7. Concurrent mutation/snapshot yields coherent state.
8. Private/sensitive fields are absent.
9. Handler queries owner once for multiple selectors in the group.
10. Restart/composition uses the production owner, not fake/default state.
11. Unavailable sibling fields remain unavailable.
12. Documentation/source matrix updates match tests.

Additional group-specific tests:

- exhaustive status code mapping;
- public RouterInfo serialization round trip;
- tunnel detail redaction;
- connection-limit configuration provenance;
- ban details only from the real owner;
- exact rolling-window semantics when present.

## 11. Verification commands

Run focused tests for each touched owner crate/module, then:

```bash
cargo check -p emissary-core
cargo test -p emissary-core
cargo clippy -p emissary-core --all-targets -- -D warnings
cargo test -p emissary-cli --no-default-features --features i2pcontrol router_info
cargo test -p emissary-cli --no-default-features --features i2pcontrol production_composition
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

If a group touches another workspace crate, run its package-scoped check/test/clippy. Do not add remote CI, network farms, coverage gates, or long soak tests.

## 12. Documentation and static guards

- Update the frozen M025 source matrix after each group.
- Record exact available/unavailable counts and owners.
- Add compile-time or unit guards for snapshot bounds and enum mappings.
- Document the absence of new samplers/pollers and the read-only ownership boundary.
- Do not mark unavailable fields implemented.

## 13. Acceptance criteria

M026 is implementation-complete when:

- every M025-feasible owner group is either implemented with bounded source evidence or explicitly rejected with a recorded reason;
- no source changes router behavior or consumes owner events;
- every newly available selector has exact shape/type, bounds, failure, and production-composition tests;
- remaining unavailable fields have no fabricated defaults and are accurately documented;
- core/CLI/package verification passes or exact unrelated blockers are recorded;
- changed files remain adjacent to named owners and I2PControl adapters;
- no historical sampler, missing tunnel, generic introspection framework, dependency, CI, or upstream action occurs;
- an implementation disposition freezes final source counts for M027.

## 14. Stop conditions

Stop a field/group and retain it as unavailable if:

- authoritative state is not currently tracked;
- exact semantics require a new rolling window/history collector;
- snapshot requires consuming an event receiver or changing task ownership;
- collection cannot be bounded without incorrect truncation;
- requested data includes private/security-sensitive state not explicitly required;
- implementation requires router algorithm or hot-path redesign;
- work expands into missing tunnels, CI, frontend, or upstream activity.

M026 completion unblocks M027 regardless of whether all fields became available, provided the final unavailable set is truthful and fully recorded.
