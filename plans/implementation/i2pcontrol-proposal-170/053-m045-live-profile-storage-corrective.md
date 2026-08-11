# M053 — M045 Corrective Live ProfileStorage Inspection Source

Status: closed

Planning baseline: `bf9c2eeb` — M045 blocked after stale startup-snapshot rollback

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Corrects:

- `plans/implementation/i2pcontrol-proposal-170/045-routerinfo-known-peer-directory.md`;
- `plans/closure/i2pcontrol-proposal-170/045-closure.md`.

Milestone class: corrective capability + containment invariant

Hard dependencies:

- M044 closed;
- M045 implementation/closure finding at `bf9c2eeb` accepted as the current blocker.

Pinned authority: I2P Proposal 170 `I2PControl Expansion`, Open, revision `2026-05-20`.

## 1. Objective

Correct the single architectural defect that blocked M045: replace the rejected startup-time `Router::inspection_snapshot()` peer-directory source with a neutral, cloneable, request-time live inspection source backed by the canonical `ProfileStorage`, then re-enable and close the three M045 Proposal 170 fields:

- `i2p.router.netdb.peers`;
- `i2p.router.netdb.peers.list`;
- `i2p.router.netdb.peers.info`.

The corrective source must observe peer-directory changes after I2PControl startup without recreating the I2PControl server or retaining `Router`, mutable NetDB authority, or a stale parallel cache.

All Proposal 170 policy remains inside `emissary-cli/src/i2pcontrol/**`: ordering, bounds, Base64/wire representation, complete-join policy, source disposition, JSON serialization, compatibility behavior, response-size enforcement, and sanitized errors.

## 2. Why M045 failed and why prior verification missed it

The M045 implementation attempt at `5ae0477` used an owned `CoreSnapshot` produced by `Router::inspection_snapshot()` when I2PControl was composed. Unit, conformance, and production-composition fixtures validated shape, bounds, failure behavior, and static containment, but did not mutate the canonical peer directory after source construction. The tests therefore proved that the snapshot was safe and internally consistent, not that it remained live.

Closure at `bf9c2eeb` correctly rejected the source because peer discovery/churn after startup was invisible and restored the source matrix to 16 available, 1 protocol-permitted neutral, and 26 unavailable.

The regression evidence added by this corrective pass must explicitly construct the inspection source first, mutate canonical `ProfileStorage` through its normal owner/test path second, and prove a later snapshot/request observes the change.

## 3. Current evidence and intended seam

Current core already contains the required canonical data and neutral inspection vocabulary:

- `RouterContext` retains a cloneable `ProfileStorage<R>`;
- `ProfileStorage::get_router_ids(Bucket::Any, ...)` provides live router enumeration inside `emissary-core`;
- `ProfileStorage::get_raw()` / `Reader::raw_router_info()` provide public serialized RouterInfo bytes;
- `ProfileStorage::reader()` provides a read-only multi-map guard for coherent lookup after enumeration;
- `emissary-core/src/inspection.rs` is already the public neutral read-only inspection boundary;
- `Router` already owns `RouterContext` and may expose a narrow inspection constructor without making `profile`, `ProfileStorage`, or `Bucket` public.

The preferred implementation is a public cloneable type in `emissary-core::inspection`, conceptually `PeerDirectoryInspection<R>`, that privately owns a clone of `ProfileStorage<R>` and exposes only a synchronous bounded snapshot operation returning owned public data. `Router` exposes one read-only constructor/accessor for this inspection type. Exact naming is implementation-local; the authority and behavior constraints are normative.

Do not make the private `profile` module public and do not publicly re-export `ProfileStorage` or `Bucket` merely to satisfy I2PControl.

## 4. Authorized production path budget

Core production changes are limited to:

- `emissary-core/src/inspection.rs`;
- `emissary-core/src/router/mod.rs`.

Composition and I2PControl changes are limited to:

- `emissary-cli/src/main.rs` as composition-only wiring;
- `emissary-cli/src/i2pcontrol/**` for the live source adapter, contract promotion, handler integration, and tests.

Documentation/test/planning updates may touch:

- `docs/i2pcontrol/**`;
- `emissary-cli/tests/**`;
- `plans/**`.

Not authorized without a new corrective disposition:

- `emissary-core/src/profile.rs`;
- `emissary-core/src/router/context.rs`;
- `emissary-core/src/netdb/**`;
- `emissary-core/src/lib.rs` public re-export expansion;
- transport/tunnel/crypto/I2NP/LeaseSet code;
- AddressBook, proxy, UI, workflows, release, or unrelated code.

If the implementation cannot meet live/coherent semantics within the two authorized core paths, stop and record the exact missing primitive rather than broadening the diff.

## 5. Invariants

1. The core inspection API is neutral: no `I2PControl`, Proposal 170, JSON-RPC, selector, or wire-key terminology in core types or methods.
2. The source is read-only. It exposes no add/discover/remove/update operation and no mutable `ProfileStorage`, NetDB, router, socket, channel, session, transport, tunnel, or command handle.
3. Only public router identity and public serialized RouterInfo bytes may cross the boundary. No private/session keys, LeaseSet private material, message payloads, or sensitive runtime objects.
4. The source is live at request time. A directory change after source construction is visible to a later snapshot without source reconstruction.
5. The source is bounded. The caller supplies an explicit item bound and oversize state fails rather than silently truncating where the field contract requires complete results.
6. Snapshot collection is synchronous and read-only; no core lock is held across `.await`, network I/O, sleep, JSON serialization, or response assembly.
7. Peer churn cannot produce fabricated completeness. If an enumerated peer loses its raw RouterInfo before the bounded snapshot can copy it, return an explicit incomplete/unavailable result or another fail-closed neutral error; do not insert an empty string or silently fabricate adjacent data.
8. Deterministic ordering, deduplication, Base64 conversion, Proposal 170 result shaping, and error translation remain in I2PControl.
9. Default/no-I2PControl router behavior is unchanged except for passive availability of the public inspection constructor; no background work, timers, polling, storage, or network operations are added.
10. No router/peer selection, scoring, discovery, NetDB query/storage, eviction, transport, tunnel, congestion, retry, cryptographic, or LeaseSet behavior changes.
11. The three M045 source rows remain unavailable until the live production source and regression fixture both exist.
12. No upstream interaction is authorized.

## 6. Work packages

### WP1 — Neutral live core inspection handle

In `emissary-core/src/inspection.rs`, add the smallest cloneable read-only peer-directory inspection type and owned snapshot DTO required by M045.

The handle may privately contain `ProfileStorage<R>` because it remains inside `emissary-core`; the public type must not expose that field or any mutable authority.

The snapshot operation must:

- enumerate the current canonical known-peer directory at call time;
- enforce the caller-provided bound before returning an unbounded wire-facing collection;
- copy owned public RouterIds and raw public RouterInfo bytes;
- define fail-closed behavior for an incomplete ID-to-RouterInfo join caused by churn;
- release all storage locks before returning;
- perform no async/network/storage writes.

Use existing `ProfileStorage` methods and `Reader` where sufficient. Do not modify `profile.rs` simply to make its internal types public.

### WP2 — Narrow Router exposure

In `emissary-core/src/router/mod.rs`, expose one read-only method that returns/clones the peer-directory inspection handle from the router's existing `RouterContext::profile_storage()` owner.

The method must not return `RouterContext`, `ProfileStorage`, `Bucket`, NetDB handles, or any mutable owner. No startup/event-loop behavior changes are permitted.

### WP3 — Replace the rejected startup snapshot composition

In `emissary-cli/src/main.rs`, compose the live inspection handle into I2PControl when the feature/server is enabled.

Do not call `Router::inspection_snapshot()` for the M045 directory source. Do not take a one-shot copy at composition time. The object retained by I2PControl must resolve current state when its snapshot operation is invoked.

Keep this wiring feature-gated and composition-only.

### WP4 — I2PControl adapter and exact field integration

Under `emissary-cli/src/i2pcontrol/**`:

- restore a peer-directory source abstraction only if useful to keep core generic details out of handlers;
- adapt the core live inspection result into I2PControl-owned DTOs;
- query the source once per M045 request group;
- deterministically sort/deduplicate as required by the existing contract;
- Base64 encode public RouterInfo only at the I2PControl/wire boundary;
- preserve the existing 10,000-item and serialized-size protections;
- preserve fail-closed behavior for incomplete joins;
- promote only the three M045 rows in `PROPOSAL_170_CONTRACT` after the live regression passes;
- update source maps/conformance expectations from 16/1/26 to 19/1/23 only after production evidence exists.

Do not use this corrective pass to implement M046 active-peer/limit fields or any later RouterInfo source.

### WP5 — Regression and containment evidence

Add the exact regression missing from M045:

1. construct canonical `ProfileStorage` with a known initial directory;
2. construct/clone the new inspection source;
3. take snapshot A and assert the initial state;
4. mutate canonical storage through its existing normal test-facing owner (`discover_router`/equivalent), including raw public RouterInfo;
5. take snapshot B from the same inspection source instance;
6. assert the newly discovered peer is present and its public RouterInfo bytes are current;
7. where practical, update an existing peer RouterInfo and prove a later snapshot reflects the update;
8. prove oversize and incomplete-join behavior fail closed;
9. prove no source reconstruction is required.

Also add/retain handler fixtures for the exact three Proposal 170 keys and a production-composition test proving the server retains the live source rather than a `CoreSnapshot` copy.

Static review must show no new public `ProfileStorage`/`Bucket` re-export and no mutable method on the inspection type.

## 7. Failure, cancellation, restart, and contention semantics

The inspection operation is synchronous and request-scoped. It has no cancellation task, worker, timer, background queue, or persistence. A dropped I2PControl request drops only owned snapshot data; it cannot cancel or mutate peer discovery.

Restart requires no migration. The handle is reconstructed from the new router instance and immediately reflects that instance's canonical storage.

Concurrent peer churn is expected. A snapshot may represent the directory at a bounded instant during collection, but it must not knowingly combine an enumerated ID set with fabricated/missing RouterInfo. If a required join cannot be completed, fail the affected request through the sanitized inspection error path. No lock may escape core or be held during I2PControl async/wire work.

## 8. Compatibility, migration, security, and authorization

No schema, disk format, configuration, authentication, TLS, token, AddressBook, tunnel-management, or compatibility-selector migration is authorized. Exact direct-presence semantics and JSON types for the three keys remain unchanged.

The inspection type is a security boundary. Its public surface should be smaller than `ProfileStorage`, not a wrapper that forwards arbitrary methods. Public DTOs contain owned public data only.

External Proposal 170/reference material is read-only evidence. Do not open or prepare upstream issues, PRs, reviews, submissions, adoption requests, merge requests, or maintainer outreach.

## 9. Verification

Run focused tests first, including the new post-construction churn regression. Then run at minimum:

```bash
cargo test -p emissary-core peer_directory --no-fail-fast
cargo check -p emissary-core
cargo test -p emissary-core --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol router_info --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition --no-fail-fast
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

Use targeted formatting only; retain the documented repository-wide stable/nightly rustfmt qualification. Do not add CI, coverage, fuzz, soak, platform-matrix, or release automation.

## 10. Acceptance criteria

M053 may close only when all of the following are evidenced:

- one neutral cloneable live peer-directory inspection source exists under `emissary-core::inspection`;
- the only core production paths changed are `inspection.rs` and `router/mod.rs`;
- no `profile.rs`, NetDB, `lib.rs` re-export, router-context, transport, tunnel, crypto, I2NP, AddressBook, proxy, or UI production change occurred;
- the same inspection source instance observes canonical peer-directory mutation after construction;
- public RouterInfo bytes are current and no stale startup `CoreSnapshot` is used for M045;
- incomplete churn joins and oversize results fail closed;
- no mutable or sensitive core authority crosses the boundary;
- the exact three M045 selectors are live, bounded, deterministic, and contract-correct;
- source accounting is truthfully 19 available, 1 protocol-permitted neutral, 23 unavailable;
- no-feature behavior and retained Proposal 170 tests pass;
- an independent closure record identifies the implementation head, exact commands/results, changed-path audit, churn regression, security review, and residual findings;
- M045 is recorded as corrected/closed only through that accepted closure; M046 is not registered ready before then.

## 11. Stop conditions

Stop and require another corrective disposition if:

- coherent live snapshots require changing `profile.rs` or NetDB behavior;
- the proposed inspection object must expose mutable storage/router authority;
- implementation requires a background cache, poller, watcher, or persistent mirror;
- snapshot collection materially changes peer discovery/storage/eviction semantics;
- the three field contracts require data not owned by `ProfileStorage`;
- a path outside the authorized production budget is necessary.

Do not weaken the live-source requirement to preserve the path budget. Record the blocker instead.

## 12. Closure evidence required

The closure record must reference M045 and its blocked closure, enumerate the stale-source defect, show the failing-before/passing-after post-construction churn regression, record exact verification outcomes, verify the two-file core path budget, inspect the public inspection surface for mutable/sensitive authority, reconcile the 43-row source matrix, and attest that all external sources were read-only and no upstream interaction or contribution artifact was created.
