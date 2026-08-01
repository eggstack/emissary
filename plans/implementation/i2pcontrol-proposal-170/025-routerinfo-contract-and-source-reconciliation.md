# M025 — RouterInfo Contract and Source Reconciliation

Status: implemented

Primary class: invariant/evidence corrective pass

Hard dependencies:

- M020 closed for implementation
- M022 closed for implementation
- M023 closed for implementation
- M024 closed for implementation

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Prior defect record:

- `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md`

## 1. Bounded objective

Rebuild the exact Proposal 170 RouterInfo contract and source matrix after the authentication, AddressBook, tunnel inventory, and SAM ownership corrections have landed.

This milestone is a reconciliation pass, not a broad core implementation pass. It must:

- derive the exact 43-selector inventory, JSON types, and presence semantics from the pinned proposal;
- map each selector to a truthful current Emissary source owner, protocol-permitted neutral value, or precise unavailable reason;
- correct existing source/type contradictions and serializers that can be fixed inside `i2pcontrol`;
- produce the bounded, owner-grouped worklist for M026;
- remove unsupported completion claims.

M025 must not add new NetDB, peer, tunnel-pool, queue, or historical telemetry sources itself except for trivial wiring to sources already exposed by M020–M024.

## 2. Current defects

The repository has a useful 43-field inventory, but availability labels and implementation paths are inconsistent. Examples include:

- address-book subscription/config selectors implemented with incorrect bare shapes while the canonical contract marks them unavailable;
- completion documentation despite many fields being explicitly unavailable or ambiguous;
- canonical serializers and compatibility/base serializers sharing names or evidence counts;
- field availability inferred from adjacent counters rather than exact source semantics;
- tests asserting current repository output without a single literal pinned-contract matrix driving all layers.

## 3. Required invariants

1. The canonical addition inventory contains exactly the 43 keys in the pinned proposal and no base/compatibility keys.
2. Every key has one exact JSON type classification.
3. Every key has one source classification: `available`, `protocol-permitted neutral`, or `unavailable` with a named reason.
4. `available` means a truthful current production source is wired and tested; a fake/test source is insufficient.
5. Aggregate counters cannot substitute for transport-, pool-, peer-, or queue-specific fields unless the semantics are identical.
6. Empty strings, zeros, arrays, and objects are not used as fabricated defaults.
7. One unavailable requested canonical selector causes the documented exact failure behavior; partial success is not returned unless the pinned contract defines it.
8. Base RouterInfo compatibility remains supported after M020 but is not counted in the 43 additions.
9. AddressBook and I2PTunnel fields consume the corrected shared sources from M022/M023.
10. RouterInfo handlers remain read-only except the exact log-clear selector.
11. No selector consumes frontend state or a single-owner event receiver.
12. Documentation, validation, dispatch, and fixtures derive from the same reviewed matrix without introducing a generic schema framework.

## 4. Explicit non-goals

- no new router/NetDB/peer/tunnel-pool snapshot implementation; that is M026;
- no new rolling-rate sampler or historical database;
- no transport algorithm changes;
- no peer categorization invented to match Java/i2pd vocabulary;
- no approximation of unavailable values;
- no missing tunnel data plane;
- no dependency, CI, release, frontend, or upstream work.

## 5. Expected file boundary

Primary files:

- `emissary-cli/src/i2pcontrol/rpc.rs`
- `emissary-cli/src/i2pcontrol/router_info_handler.rs`
- `emissary-cli/src/i2pcontrol/router_info.rs`
- `emissary-cli/src/i2pcontrol/production.rs`
- address-book/tunnel serializers only where corrected sources are consumed;
- conformance/source-map tests and documentation.

No `emissary-core` production change is authorized in M025.

## 6. Required work packages

### WP1 — Pinned contract table

Create/review one machine-readable table with, for every key:

- exact key string;
- exact return JSON type;
- nullable/optional semantics where stated;
- mutation/read-only behavior;
- source owner group;
- current source status;
- exact serializer function;
- focused fixture identifier.

The table may remain Rust constants plus a documentation generator test only if that is simpler than adding a new format. Do not add code generation.

### WP2 — Canonical/base/compatibility separation

Maintain separate inventories for:

- base existing I2PControl RouterInfo selectors;
- 43 canonical Proposal 170 additions;
- Emissary compatibility aliases/forms.

Validation must accept the appropriate direct base/canonical set after `Token` removal. Static coverage counts must never combine the sets.

### WP3 — Source adjudication

For each selector, inspect current production ownership and assign:

- `Available`: exact source and adapter already exist;
- `Neutral`: proposal explicitly permits null or another neutral representation and the absence is truthful;
- `Unavailable`: source absent, not authoritative, unbounded, or semantically different.

Unavailable reason vocabulary should be concise and owner-oriented, for example:

- `no rolling 15s transit source`;
- `no bounded participating tunnel detail snapshot`;
- `no transport-specific v6 error code mapping`;
- `no authoritative ban owner`;
- `no bounded active peer RouterInfo snapshot`.

Do not mark protocol ambiguity as available. Resolve wording/type ambiguity against the pinned reference implementation when possible; otherwise retain an explicit adjudication record.

### WP4 — Correct local serializers

Within existing sources, correct:

- exact field names and JSON numeric/string/object/list types;
- total transit versus received/sent semantics;
- log messages list and clear result;
- I2PTunnel quick-info shape from M023;
- address-book list, subscription, and config object shapes from M022;
- status/rate percentage units and zero-total behavior;
- nullable identity/info/clock-skew behavior.

Do not touch fields whose source is not yet exposed.

### WP5 — Request-level failure behavior

- Validate all requested direct keys before querying sources.
- If any key is unavailable, return one sanitized deterministic method failure without querying unrelated expensive groups.
- Query each available owner group at most once per request.
- Enforce result count/byte bounds after actual serialization, not only coarse pre-estimates.
- Preserve exact requested-key-only response behavior.

### WP6 — M026 source worklist

For every unavailable field, decide whether a bounded adjacent snapshot is feasible without router behavior change.

Classify:

- `M026 feasible`: current authoritative state exists and only a read-only DTO/handle is missing;
- `deferred unavailable`: state is not tracked, requires new historical sampling, requires semantic invention, or requires broad owner redesign;
- `out of scope`: tied to missing tunnel data planes or unrelated functionality.

Group feasible work by owner so M026 remains one coherent bounded inspection pass and can stop cleanly per group.

### WP7 — Claim correction

Update support documents to state exact counts:

- wire recognized;
- source available;
- neutral;
- unavailable;
- runtime implemented/unsupported where relevant.

Do not use `implemented` without naming the dimension.

## 7. Failure, cancellation, restart, and contention semantics

- RouterInfo remains read-only except log clear.
- Source query failure aborts the request; no partial result is returned.
- Snapshot adapters clone bounded state without locks across await points.
- Restart-retained fields are captured once from validated startup values.
- Live counters are read coherently enough for one request group; exact simultaneity across independent groups is not required, but no derived sum may double count.
- Log clear uses the existing owner and does not race into secret disclosure.

## 8. Focused tests

Required tests:

1. Exact 43-key set equality and uniqueness.
2. Exact JSON type table equality against literal fixtures.
3. Base/canonical/compatibility inventories are disjoint where names differ and intentionally aliased where exact names overlap.
4. Standard `Token` is removed before direct selector validation.
5. Requested-key-only output.
6. Available selectors emit exact type/shape.
7. Neutral nullable fields emit null only when permitted.
8. Every unavailable selector returns deterministic failure and no fabricated value.
9. Mixed available/unavailable request fails before expensive source reads.
10. Owner group queried at most once.
11. AddressBook subscription/config exact object fixtures.
12. I2PTunnel quick-info exact list/object fixture from shared inventory.
13. Transit, total success rate, logs, and log-clear semantics.
14. Actual serialized response bound catches underestimated payloads.
15. Documentation/source-map counts equal the machine-readable table.

## 9. Verification commands

```bash
cargo test -p emissary-cli --no-default-features --features i2pcontrol router_info
cargo test -p emissary-cli --no-default-features --features i2pcontrol conformance_manifest
cargo test -p emissary-cli --no-default-features --features i2pcontrol source_map
cargo test -p emissary-cli --no-default-features --features i2pcontrol address_book
cargo test -p emissary-cli --no-default-features --features i2pcontrol i2ptunnel
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

No core tests are required because M025 must not change core production code.

## 10. Documentation and static guards

Update:

- `docs/i2pcontrol/router-info.md`;
- `docs/i2pcontrol/router-info-source-map.md`;
- `docs/i2pcontrol/proposal-170-support.md`;
- conformance documentation and implementation-plan status.

Static guards must fail when:

- canonical count changes from 43 without an explicit proposal revision update;
- a selector lacks type/source/serializer/fixture metadata;
- documentation counts diverge;
- base or compatibility entries are counted as canonical.

Do not generate a large evidence artifact.

## 11. Acceptance criteria

M025 is implementation-complete only when:

- exact 43-key/type inventory is revalidated against the pinned source;
- every selector has one truthful source disposition;
- local serializer contradictions are corrected;
- unavailable requests never return fabricated defaults;
- M026 has a precise owner-grouped feasible source list and explicit deferred list;
- documentation uses dimension-specific counts and no closure claim;
- package verification passes or exact unrelated blockers are recorded;
- no core source implementation, sampler, dependency, CI, or upstream action occurs;
- an implementation disposition freezes the matrix and M026 input.

## 12. Stop conditions

Stop if:

- resolving a field requires inventing semantics not present in the proposal/reference;
- a source labeled available cannot be traced to one production owner;
- local reconciliation expands into core inspection implementation;
- documentation pressure encourages counting unavailable/compatibility items as complete;
- any missing tunnel data plane, CI, or upstream activity enters scope.

## 13. Frozen M026 source worklist

M025 adjudicated all 26 unavailable additions. No field currently meets the
`M026 feasible` definition because every candidate requires either a missing
bounded owner snapshot, a new historical sampler, or semantic mapping that is
not present in Emissary. M026 is therefore unblocked for a bounded owner audit
and may close the fields as deferred/out of scope without adding production
sources.

| Owner group | Fields | M026 disposition |
|---|---|---|
| `traffic-metrics` | `net.bw.transit.15s`, `net.tunnels.successrate` | deferred unavailable; exact rolling windows are not tracked and adding history is out of scope |
| `network` | `net.status.v6`, `net.error`, `net.error.v6`, `net.testing`, `net.testing.v6` | deferred unavailable; existing firewall statuses do not provide the pinned integer/error/testing mappings |
| `tunnel-pool` | participating details, exploratory/client counts/details, `queue`, `tbmqueue` | out of scope; no bounded pool/queue owner is exposed and adding one would change ownership boundaries |
| `netdb` | `netdb.peers`, `activepeers.info` | deferred unavailable; no bounded current NetDB/RouterInfo snapshot owner exists |
| `peer-limits` | `netdb.ntcp.limit`, `netdb.ssu.limit` | deferred unavailable; no authoritative transport-limit owner is exposed |
| `ban-list` | `netdb.bannedpeers` | deferred unavailable; no authoritative ban owner exists |
| `peer-list` | active/known peer lists and peer info | deferred unavailable; no bounded peer snapshot owner exists |
| `peer-stats` | `netdb.activepeers.stats` | deferred unavailable; no bounded per-peer statistics owner exists |

There are no M026-feasible fields in the frozen matrix. Any future source
implementation requires a new owner-specific plan or an explicit update to
M026; it must not be inferred from aggregate metrics or fabricated defaults.

M025 closure unblocks M026.
