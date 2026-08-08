# M045 — RouterInfo Known-Peer Directory Sources

Status: blocked — corrected only through M053

Planning baseline: `b759038`

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Corrective authority: `plans/implementation/i2pcontrol-proposal-170/053-m045-live-profile-storage-corrective.md`

Milestone class: capability + containment invariant

Hard dependency: M044 closed; corrective dependency M053 must close before M045 can be accepted

Pinned authority: I2P Proposal 170 `I2PControl Expansion`, Open, revision `2026-05-20`.

## 1. Objective

Make the three canonical known-peer RouterInfo additions truthful and operational without adding a new core owner:

- `i2p.router.netdb.peers`;
- `i2p.router.netdb.peers.list`;
- `i2p.router.netdb.peers.info`.

Use the existing `ProfileStorage` owned by `RouterContext` as the canonical known-router directory. Keep selection, ordering, output bounds, Base64/public-RouterInfo serialization, error translation, Proposal 170 source disposition, and JSON-RPC behavior inside `emissary-cli/src/i2pcontrol/**`.

## 2. Current evidence

`Router::inspection_snapshot()` demonstrates that core can enumerate known router IDs and retrieve public serialized RouterInfo without mutable NetDB authority, but M045 closure proved that retaining this one-shot snapshot at I2PControl startup is not a live source.

The M045 implementation attempt at `5ae0477` passed shape/bounds/conformance tests but was rejected at closure because later peer-directory churn was invisible. The rollback/planning head `bf9c2eeb` restored all three fields to unavailable and recorded the exact missing primitive.

`router_info_handler.rs` owns wire serialization and `rpc.rs::router_info_keys::PROPOSAL_170_CONTRACT` currently marks these three fields unavailable.

## 3. Invariants

1. M045 itself does not authorize core production changes; its disproven zero-core assumption is corrected only by M053's explicitly bounded exception.
2. No new NetDB command, query, polling loop, cache, or mutable handle is introduced.
3. The source is read-only and contains public RouterInfo only; no keys, LeaseSet private material, sockets, channels, or mutable subsystem objects cross the boundary.
4. Results are bounded before wire serialization and deterministic after collection.
5. A missing raw RouterInfo for an enumerated ID fails closed according to the exact field contract; it is not replaced by an empty string or adjacent data.
6. Direct Proposal 170 presence semantics and compatibility-mode behavior do not change.
7. Default/no-I2PControl execution performs no new background work.
8. A source retained across I2PControl startup must remain live: canonical peer-directory mutation after source construction must be visible to a later request.

## 4. Explicit non-goals

- active-peer sources, connection limits, active-peer statistics, bans;
- NetDB protocol or storage behavior changes;
- peer discovery, scoring, routing, profiling, or eviction changes;
- new background sampling;
- public export of `ProfileStorage`/`Bucket` merely for I2PControl;
- tunnel, transport, AddressBook, frontend, CI/release, or upstream work.

## 5. Original production budget and corrective disposition

The original M045 preferred production budget was:

- `emissary-cli/src/i2pcontrol/router_info.rs`;
- `emissary-cli/src/i2pcontrol/router_info_handler.rs`;
- `emissary-cli/src/i2pcontrol/production.rs`;
- `emissary-cli/src/i2pcontrol/rpc.rs`;
- `emissary-cli/src/i2pcontrol/server.rs` as composition/state plumbing only;
- `emissary-cli/src/main.rs` as composition only.

That zero-core budget reached its stop condition. M053 now separately authorizes exactly:

- `emissary-core/src/inspection.rs`;
- `emissary-core/src/router/mod.rs`.

M053 does not authorize `profile.rs`, `router/context.rs`, NetDB, or `lib.rs` public re-export changes. M045 must not be implemented independently of M053 or by restoring the rejected startup `CoreSnapshot` approach.

## 6. Required corrective behavior

M053 owns the corrective implementation details. For M045 acceptance, the resulting production path must:

- retain a neutral cloneable live core inspection source rather than a one-shot startup snapshot;
- observe current canonical `ProfileStorage` state at request time;
- return only bounded owned public RouterIds and serialized public RouterInfo;
- leave sorting/deduplication, Base64/wire conversion, bounds, Proposal 170 source disposition, and JSON-RPC behavior in I2PControl;
- fail closed for incomplete churn joins;
- promote only the three M045 rows after live-source regression evidence passes.

## 7. Failure, cancellation, restart, and contention

The operation is request-scoped and read-only. No locks may be held across `.await`, network I/O, sleep, or JSON serialization. Source failure aborts the RouterInfo request through the existing sanitized inspection error path; no partial peer result is fabricated. Restart requires no migration or persisted state.

Concurrent peer churn is normal. A request-time snapshot may reflect a bounded instant during collection, but if an enumerated peer's required public RouterInfo cannot be copied coherently, the request fails closed rather than returning a plausible partial directory.

## 8. Compatibility and migration

No storage/schema/configuration migration. Base nested `Selector` behavior remains unchanged. Canonical direct fields retain exact Proposal 170 spelling and types.

## 9. Tests and verification

The original fixtures remain necessary but insufficient. M053 adds the missing regression:

1. construct the live source;
2. observe initial state;
3. mutate canonical `ProfileStorage` through its existing normal owner/test path;
4. query the same source instance again;
5. prove the new/current peer and RouterInfo are visible without source reconstruction.

Also retain zero/one/many peer cases, deterministic ordering, bound rejection, known ID with public RI, failure propagation, conformance-manifest fixtures, production composition, no-feature checks, and the bounded package matrix defined by M053.

## 10. Acceptance criteria

M045 may close only through an accepted M053 closure showing that:

- all three fields are served from the live canonical known-peer directory;
- the same retained inspection source observes canonical mutation after construction;
- exact JSON types/shapes match the pinned contract;
- bounds and incomplete-join failure semantics are explicit;
- no fabricated value path or stale `CoreSnapshot` source remains;
- the only corrective core production changes are the two paths authorized by M053;
- no-feature behavior remains unchanged;
- source accounting is truthfully 19 available, 1 protocol-permitted neutral, and 23 unavailable;
- the independent closure records exact implementation head, verification outcomes, changed-path audit, and security review.

The original M045 requirement of zero `emissary-core/**` production diff is superseded only by M053's explicit two-file corrective authorization; it is not a general relaxation of the roadmap containment rule.

## 11. Stop conditions

Stop and require another corrective disposition if the only implementation path requires changing peer discovery/routing behavior, modifying `profile.rs` or NetDB semantics, broadly exporting `ProfileStorage`/`Bucket`, retaining private material, exposing mutable authority, adding a background cache/poller, or touching core production paths outside M053's explicit budget.

M045 previously reached its stop condition because `RouterContext::profile_storage()` exposed the canonical owner but `emissary-cli` could not enumerate it without the private `Bucket` type. The rejected `Router::inspection_snapshot()` adapter was startup-stale. M053 is now the authorized correction for exactly this gap.

## 12. Closure evidence required

Closure must reference M053, the blocked M045 closure, and the stale-source implementation attempt; include a requirement-to-evidence matrix; show the failing-before/passing-after post-construction churn regression; record exact verification commands/results; audit the two-file core corrective budget; reconcile all three field dispositions; and attest that external specifications/reference implementations were read-only with no upstream issue, PR, review, submission, adoption, merge, maintainer contact, or contribution artifact.
