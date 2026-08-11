# I2PControl Proposal 170 Milestone M050 — Closure Status

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/050-routerinfo-network-state-sources.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Repository baseline reviewed: `694f06b`

Implementation commit:

- `11b1d33` — `i2pcontrol: expose network state sources`

Closure date: 2026-08-11

Pinned contract: Proposal 170, `I2PControl Expansion`, Open, revision
`2026-05-20`.

## 1. Executive finding

M050 is closed internally against the pinned Proposal 170 revision. All five
planned RouterInfo additions now have a truthful request-time source and exact
integer serializers:

- IPv6 status is read from the independently cached transport reachability
  state;
- IPv4 and IPv6 error values map only independently known neutral reasons, and
  currently return the truthful `None` reason because Emissary has no canonical
  error owner;
- IPv4 and IPv6 testing values reflect active existing SSU2 peer tests;
- Proposal 170/i2pd numeric mapping exists only in I2PControl;
- no new probe, timer, network request, or router behavior was introduced.

The RouterInfo matrix is now 40 available, 1 protocol-permitted neutral, and 2
unavailable. The remaining unavailable rows are router news and banned peers,
which remain M051 scope.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| Separate v4/v6 status | `EventHandle` status caches; `TransportManager::on_firewall_status`; production `EventHandleMetrics`; exact v6 fixture | pass | Existing firewall transitions remain the authoritative status source |
| Independent v4/v6 error state | Neutral `NetworkState` and `NetworkErrorReason` in `emissary-core/src/inspection.rs`; separate atomic slots and setters in `events.rs` | pass | No error is inferred from `FirewallStatus`; no unsupported cause is fabricated |
| Independent v4/v6 testing state | `PeerTestManager::publish_testing_state()` at existing peer-test maintenance transitions; separate event-handle booleans | pass | Observes existing SSU2 peer tests and adds no reachability traffic |
| Exact status mapping | `network_status_code()` and `p170.status_v6.integer` fixture | pass | Supported production states map OK=0, Firewalled=1, Unknown=2; symmetric NAT has no distinct i2pd status and remains Unknown |
| Exact error mapping | `network_error_code()` and `p170.error_v4.integer`/`p170.error_v6.integer` fixture | pass | None=0, ClockSkew=1, Offline=2, SymmetricNAT=3, FullConeNAT=4, NoDescriptors=5 |
| Exact testing mapping | Handler fixture for both families | pass | `true`/`false` serialize as integer 1/0 |
| Direct presence and integer JSON types | `handle_router_info_network_state_wire_fixture`; contract manifest rows | pass | Selector values are ignored, including false/null/non-boolean values |
| One coherent request source | `assemble_response()` acquires one network snapshot for base and M050 selectors | pass | No duplicate source query for combined selectors |
| Source accounting | `PROPOSAL_170_CONTRACT`, literal fixtures, conformance manifest, and source map | pass | 40 available / 1 neutral / 2 unavailable |

## 3. Production implementation evidence

The implementation remains split at the existing ownership boundary:

- `emissary-core/src/inspection.rs` defines only neutral state and error-reason
  DTOs; it contains no Proposal 170 keys or wire integers.
- `emissary-core/src/events.rs` stores short-lived atomic status, optional
  neutral error, and testing state for each address family.
- `emissary-core/src/transport/mod.rs` continues to publish status only at the
  existing firewall transition.
- `emissary-core/src/transport/ssu2/peer_test/mod.rs` publishes whether the
  already-running local v4/v6 peer-test sets are active.
- `emissary-cli/src/i2pcontrol/production.rs` adapts neutral state into the
  control-plane snapshot.
- `emissary-cli/src/i2pcontrol/router_info_handler.rs` owns numeric mapping,
  exact integer JSON serialization, selector presence, and source grouping.
- `emissary-cli/src/i2pcontrol/rpc.rs` changes only the five M050 contract rows
  from unavailable to available and records their fixtures.

No transport algorithm, peer selection, handshake, retransmission, tunnel
building, routing, NetDB, cryptographic, I2NP, LeaseSet, or data-plane behavior
changed. No persistent store, background sampler, new task, network probe, or
administrative mutation was added.

## 4. Verification executed

### Commands run

```bash
rtk cargo check -p emissary-core
rtk cargo check -p emissary-cli --no-default-features
rtk cargo check -p emissary-cli --no-default-features --features i2pcontrol
rtk cargo test -p emissary-core network_state_tracks_families_errors_and_testing_independently -- --exact
rtk cargo test -p emissary-core inspection
rtk cargo test -p emissary-core
rtk cargo test -p emissary-cli --no-default-features
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m027_literal_fixtures
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test router_info_truthfulness
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test static_guards
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol
rtk cargo clippy -p emissary-core --all-targets -- -D warnings
rtk cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings
rtk cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
rtk git diff --check
rtk cargo fmt --all -- --check
```

### Results

- Core and both CLI configuration checks passed.
- The focused core network-state test passed.
- Core inspection tests passed: 7 tests.
- Core package tests passed.
- CLI no-feature tests passed.
- Conformance manifest, literal fixture, RouterInfo truthfulness, and static
  guard suites passed; the static guard suite reported 38 tests.
- Full CLI I2PControl-feature tests passed.
- Core and both CLI clippy invocations passed with `-D warnings`.
- `git diff --check` passed.
- One attempted test command supplied the `cargo-nextest`-specific
  `--no-fail-fast` option to `cargo test`; cargo rejected that option. The same
  suites were rerun without the unsupported option and passed.
- The formatter check remains blocked only by pre-existing repository-wide
  stable/nightly rustfmt differences in unrelated files. No formatter churn was
  retained.

## 5. Invariant review

1. Core state is protocol-neutral; all Proposal 170 names, integer mappings,
   JSON types, and compatibility semantics remain in I2PControl.
2. v4 and v6 are separate throughout status, error, and testing state.
3. Unknown status remains Unknown; it is not rewritten as testing, offline, or
   success.
4. Firewall status does not infer an error or testing value. Error defaults to
   no known reason, and testing is published only by the peer-test owner.
5. Publication is passive and best-effort; it cannot fail a transport operation.
6. No lock spans I/O, serialization, sleep, cancellation, or await. Reads use
   atomic copies and the peer-test publication has no await.
7. The no-feature CLI path performs no I2PControl work and remains green.
8. Changed core paths stay within the M050 machine-readable budget: events,
   inspection, transport, and SSU2 peer-test reachability ownership.

## 6. Failure and recovery review

State is process-local and starts as status Unknown, error None, and testing
false. Restart therefore reports truthful unknown/not-testing state until an
existing owner publishes a stronger fact. Atomic publication cannot perturb
transport progress. Peer-test expiration removes stale tests before the owner
publishes current testing state; separate v4/v6 active sets can enter and exit
independently. An unavailable error reason is represented as code 0 rather than
an adjacent firewall result. No request waits for a probe or a state change.

## 7. Migration and compatibility review

No schema, persistence, configuration, or migration changes were introduced.
The five exact selector names, direct presence semantics, integer JSON types,
authentication, TLS, and compatibility-mode behavior remain intact. Existing
base network status strings continue to use their existing serializers. No
rollback migration is required.

## 8. Security review

The fields remain behind authenticated I2PControl dispatch and sanitized
read-only response assembly. No private key, session key, socket, channel,
mutable transport/session handle, or message payload crosses the inspection
boundary. State is bounded to a fixed set of atomics and booleans, with no new
request amplification or network surface.

## 9. Documentation and operations

Updated the RouterInfo source map, RouterInfo documentation, Proposal 170
support/conformance documentation, implementation handoff README, contract
fixtures, and active roadmap/registry counts. The source map records the exact
status/error/testing code table and all five fixtures. There is no new operator
task: restart resets process-local observations, and normal transport/peer-test
transitions repopulate them.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | `cargo fmt --all -- --check` is not green under the available formatter because the checked-in repository contains nightly-only rustfmt settings and unrelated pre-existing formatting differences | Global formatting evidence remains qualified | Preserve the qualification and avoid unrelated formatter churn |

No medium, high, or critical implementation findings remain. Router news and
banned peers are explicitly retained as M051 scope, not unresolved M050 defects.

## 11. Roadmap disposition

M050 is closed and its hard dependency is satisfied for M051. M051 is now the
sole dependency-ready handoff. M052 remains blocked until M051 receives an
accepted disposition. The broader Proposal 170 roadmap remains partial and
internal-only; the two remaining unavailable RouterInfo rows are not fabricated
as supported.

## 12. Registry updates

The following records were updated with this closure:

- M050 implementation plan status changed to `closed`;
- M051 implementation plan status changed to `ready`;
- `plans/registry.md` now lists M050 closed and M051 as the sole ready plan;
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` now records M050 closed,
  M051 ready, and only news/banned-peer rows remaining;
- `plans/implementation/i2pcontrol-proposal-170/README.md` now reflects the
  M051 handoff and current source sequence;
- current RouterInfo documentation and source accounting now record 40/1/2.

The only external authority consulted was read-only Proposal 170/i2pd reference
material for contract semantics and numeric vocabulary. No upstream repository,
issue, pull request, review, discussion, submission, adoption request, merge,
maintainer channel, or contribution artifact was mutated or prepared. The
implementation and planning commits are confined to the authorized internal
`eggstack/emissary` repository.

Disposition: **closed**.
