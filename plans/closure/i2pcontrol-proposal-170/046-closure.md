# I2PControl Proposal 170 Milestone M046 — RouterInfo Active-Peer Inventory and Limits Closure

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/046-routerinfo-active-peer-inventory-and-limits.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md#milestone-046--active-peer-inventory-and-limits--closed`

Implementation commit: `fca7a5f` (`i2pcontrol: add active peer inventory and transport limits`).

Review date: 2026-08-10

Pinned Proposal 170 revision: `2026-05-20`, Open; unchanged during implementation.

## 1. Final disposition

M046 is closed. The four fields now use live bounded sources:

- `i2p.router.netdb.activepeers.list` reads current connected peer IDs;
- `i2p.router.netdb.activepeers.info` joins those IDs to the live M045 public
  RouterInfo directory and emits Base64 public RouterInfo values;
- `i2p.router.netdb.ntcp.limit` and `i2p.router.netdb.ssu.limit` read the
  authoritative finite `max_connections` values from the transport manager's
  retained configuration.

The source matrix moved from 19 available / 1 protocol-permitted neutral / 23
unavailable to 23 available / 1 protocol-permitted neutral / 19 unavailable.
The broader Proposal 170 implementation remains partial.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Neutral transport snapshot | `emissary-core/src/inspection.rs` defines cloneable `TransportInspection` and owned `TransportInspectionSnapshot` containing only peer IDs and optional finite limits | pass |
| Current connected peers | `emissary-core/src/transport/mod.rs` copies sorted IDs after connection establishment and removal; the handle is request-time live | pass |
| Authoritative limits | The source copies `Ntcp2Config::max_connections` and `Ssu2Config::max_connections`; finite `NonZeroUsize` values become integers, while `None` remains unavailable | pass |
| Narrow Router exposure | `Router::transport_inspection()` returns only the cloneable read-only handle; no `Router`, manager, socket, session, channel, key, or control object crosses the seam | pass |
| Active list ordering and bounds | `canonical_active_peer_inventory_and_limits_return_exact_wire_values`; production source bound is 10,000 and handler rechecks the bound, sorts, and deduplicates | pass |
| Active RouterInfo join | The same fixture proves active IDs join to M045 public bytes and serialize as `AQI=`/`AwQ=` in deterministic order | pass |
| Churn/incomplete join | `active_peer_router_info_join_fails_closed_on_source_churn` returns a sanitized internal error when an active ID has no public directory entry; no RouterInfo is fabricated | pass |
| Empty active population | The owned snapshot supports an empty authoritative list; no fallback/default source is used | pass |
| Finite NTCP/SSU limits | The exact-wire fixture returns `64` and `128` as JSON integers | pass |
| Unlimited/disabled semantics | `unlimited_transport_limit_is_unavailable_not_a_sentinel`; `None` is rejected with an explicit finite-limit-unavailable error | pass |
| No live/control handles in DTO | `transport_inspection_handle_contains_only_owned_snapshot_state` statically checks the handle's field block for the single owned snapshot field and forbids socket/session/channel/key/control types | pass |
| Core seam behavior | `inspection::tests::transport_inspection_is_cloneable_bounded_and_live` proves clones observe updates and item bounds reject oversize snapshots | pass |
| Feature composition | `main.rs` composes `LiveActivePeerSource` only in the I2PControl setup path, alongside the existing M045 live directory source | pass |
| Exact source accounting/docs | `PROPOSAL_170_CONTRACT`, source-map, literal fixtures, and conformance fixtures agree on 23/1/19 and the four available rows | pass |
| Core containment | Core production changes are limited to `inspection.rs`, `transport/mod.rs`, and `router/mod.rs`, exactly the M046 exception budget | pass |

## 3. Exact changed-path audit

Production and composition paths:

- `emissary-cli/src/i2pcontrol/production.rs`;
- `emissary-cli/src/i2pcontrol/router_info.rs`;
- `emissary-cli/src/i2pcontrol/router_info_handler.rs`;
- `emissary-cli/src/i2pcontrol/rpc.rs`;
- `emissary-cli/src/i2pcontrol/server.rs`;
- `emissary-cli/src/main.rs`;
- `emissary-core/src/inspection.rs`;
- `emissary-core/src/transport/mod.rs`;
- `emissary-core/src/router/mod.rs`.

Tests and documentation:

- `emissary-cli/tests/conformance_manifest.rs`;
- `emissary-cli/tests/m027_literal_fixtures.rs`;
- `emissary-cli/tests/router_info_truthfulness.rs`;
- `emissary-cli/tests/static_guards.rs`;
- `docs/i2pcontrol/router-info-source-map.md`.

No crypto, I2NP, NetDB protocol/storage, tunnel, routing, transport state
machine, proxy, UI, AddressBook, workflow, release, or upstream path changed.

## 4. Verification executed

All commands passed unless noted:

- `cargo test -p emissary-core inspection --no-fail-fast`;
- `cargo test -p emissary-core --no-fail-fast`;
- `cargo test -p emissary-cli --no-default-features --no-fail-fast`;
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol router_info --no-fail-fast`;
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast`;
- `cargo clippy -p emissary-core --all-targets -- -D warnings`;
- `cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings`;
- `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings`;
- `git diff --check`.

`cargo fmt --all` was run. The repository's configured unstable rustfmt options
are unavailable on the installed stable toolchain, so it emitted the known
nightly-option warnings; unrelated formatter-only changes were removed and no
formatting churn is included in the implementation commit.

## 5. Failure, restart, and contention review

Transport facts are copied under a short core read lock into owned DTOs. No
lock is retained across I2PControl awaits or serialization. Connection churn
updates the shared snapshot on the existing establish/close transitions and
does not add a worker, timer, poller, or persistence. A missing public
RouterInfo during the active-ID join fails the request deterministically. An
empty active snapshot is a successful authoritative zero-peer result. Restart
reconstructs the source with no migration or persisted state.

No admission, timeout, congestion, handshake, routing, discovery, or transport
algorithm behavior changed.

## 6. Compatibility and security review

The four selectors retain exact direct-presence selection and array/integer
wire types. Legacy `i2p.router.peers.*` behavior remains separate. Unlimited
and disabled transports are not mapped to `0`, `-1`, or another sentinel.
Only public peer IDs, public serialized RouterInfo bytes, and finite configured
limits cross the boundary; no private or live transport material is exposed.

## 7. Future-plan disposition

M047 is now `ready` as the sole dependency-ready successor because its hard
dependency M046 is closed. M048, M049, M050, M051, and M052 remain `blocked`
behind their named hard dependencies. No other future plan became unblocked.
The roadmap, implementation README, registry, and source-count documentation
were updated accordingly.

## 8. Internal-only attestation

Proposal 170 and reference material were accessed read-only. No upstream
repository or maintainer channel was mutated; no upstream issue, pull request,
review, submission, adoption, merge, maintainer contact, or contribution
artifact was created or requested.

**Disposition: M046 closed; M047 ready; RouterInfo source matrix 23/1/19.**
