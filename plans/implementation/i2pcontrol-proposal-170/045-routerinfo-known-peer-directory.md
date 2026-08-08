# M045 — RouterInfo Known-Peer Directory Sources

Status: ready

Planning baseline: `b759038`

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Milestone class: capability + containment invariant

Hard dependency: M044 closed

Pinned authority: I2P Proposal 170 `I2PControl Expansion`, Open, revision `2026-05-20`.

## 1. Objective

Make the three canonical known-peer RouterInfo additions truthful and operational without adding a new core owner:

- `i2p.router.netdb.peers`;
- `i2p.router.netdb.peers.list`;
- `i2p.router.netdb.peers.info`.

Use the existing `ProfileStorage` owned by `RouterContext` as the canonical known-router directory. Keep selection, ordering, output bounds, Base64/public-RouterInfo serialization, error translation, Proposal 170 source disposition, and JSON-RPC behavior inside `emissary-cli/src/i2pcontrol/**`.

## 2. Current evidence

`Router::inspection_snapshot()` already demonstrates that `router_context().profile_storage()` can enumerate known router IDs and retrieve public serialized RouterInfo without mutable NetDB authority. The existing snapshot is not a live I2PControl source and must not become a second cache.

`router_info.rs` already defines the peer-directory DTO/trait vocabulary and `router_info_handler.rs` already owns wire serialization. `rpc.rs::router_info_keys::PROPOSAL_170_CONTRACT` currently marks these three fields unavailable.

## 3. Invariants

1. No `emissary-core/**` production change is expected or authorized for M045.
2. No new NetDB command, query, polling loop, cache, or mutable handle is introduced.
3. The source is read-only and contains public RouterInfo only; no keys, LeaseSet private material, sockets, channels, or mutable subsystem objects cross the boundary.
4. Results are bounded before allocation/serialization and deterministic after collection.
5. A missing raw RouterInfo for a known ID is represented according to the exact field contract; it is not replaced by an empty string or adjacent data.
6. Direct Proposal 170 presence semantics and compatibility-mode behavior do not change.
7. Default/no-I2PControl execution performs no new work.

## 4. Explicit non-goals

- active-peer sources, connection limits, active-peer statistics, bans;
- NetDB protocol or storage changes;
- peer discovery, scoring, routing, profiling, or eviction changes;
- new background sampling;
- tunnel, transport, AddressBook, frontend, CI/release, or upstream work.

## 5. Required production changes

Preferred production budget:

- `emissary-cli/src/i2pcontrol/router_info.rs`;
- `emissary-cli/src/i2pcontrol/router_info_handler.rs`;
- `emissary-cli/src/i2pcontrol/production.rs`;
- `emissary-cli/src/i2pcontrol/rpc.rs`;
- `emissary-cli/src/i2pcontrol/server.rs` as composition/state plumbing only;
- `emissary-cli/src/main.rs` only to provide the already-existing read-only router/profile source to I2PControl.

If implementation appears to require a change under `emissary-core/**`, stop M045 and record the exact missing public read-only primitive. Do not broaden the milestone silently.

## 6. Work packages

### WP1 — Source contract

Define one I2PControl-owned read-only peer-directory abstraction that can return a bounded generation/snapshot of known router IDs and public RouterInfo bytes. The production adapter may retain a clone of the existing `ProfileStorage` or another existing read-only core object; it must not retain `Router` itself.

### WP2 — Production adapter

Implement the source against current `ProfileStorage` APIs. Acquire/copy only the requested bounded data. Canonical ordering and duplicate handling belong to I2PControl.

### WP3 — Wire integration

Wire the source into `ProductionRouterInfoControl`. Reuse existing serializers. Change only these three `PROPOSAL_170_CONTRACT` rows from unavailable to available after focused source tests pass.

### WP4 — Evidence and guards

Add tests proving boundedness, deterministic ordering, public RouterInfo fidelity, unavailable/error behavior when the source cannot satisfy a request, and no core production diff.

## 7. Failure, cancellation, restart, and contention

The operation is request-scoped and read-only. No locks may be held across `.await`, network I/O, sleep, or JSON serialization. Source failure aborts the RouterInfo request through the existing sanitized inspection error path; no partial peer result is returned. Restart requires no migration or persisted state.

## 8. Compatibility and migration

No storage/schema/configuration migration. Base nested `Selector` behavior remains unchanged. Canonical direct fields retain exact Proposal 170 spelling and types.

## 9. Tests and verification

Focused tests must cover zero peers, one peer, deterministic many-peer ordering, bound rejection rather than truncation where required, known ID with public RI, and failure propagation.

Run at minimum:

```bash
cargo test -p emissary-cli --no-default-features --features i2pcontrol router_info --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition --no-fail-fast
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
git diff --check
```

Use targeted formatting only; do not add CI/fuzz/soak infrastructure.

## 10. Acceptance criteria

M045 may close only when all three fields are served from the live canonical known-peer directory, their exact JSON types/shapes match the pinned contract, bounds are explicit, no fabricated value path exists, no `emissary-core/**` production file changed, no-feature behavior remains unchanged, and an independent closure record records the exact implementation head and command results.

## 11. Stop conditions

Stop and require a corrective/new plan if the only implementation path requires changing peer discovery/routing behavior, exporting mutable NetDB authority, retaining private material, or modifying core production code.

## 12. Closure evidence required

Closure must include a requirement-to-evidence matrix, changed-path review, focused contract fixtures for all three keys, no-feature evidence, failure/bounds evidence, and an internal-only attestation. External specifications/reference implementations are read-only; no upstream issue, PR, review, submission, adoption, or merge activity is authorized.
