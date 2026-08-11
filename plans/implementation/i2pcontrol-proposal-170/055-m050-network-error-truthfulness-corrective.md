# M055 — M050 Corrective Network-Error Truthfulness

Status: blocked — M054 closure required first

Planning baseline: `970252c` — merged M053–M052 implementation/reclosure head

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Corrects:

- `plans/implementation/i2pcontrol-proposal-170/050-routerinfo-network-state-sources.md`;
- `plans/closure/i2pcontrol-proposal-170/050-closure.md`;
- the affected network-error findings in `plans/closure/i2pcontrol-proposal-170/052-closure.md`.

Milestone class: corrective truthfulness + containment cleanup

Hard dependencies:

- M054 accepted closure;
- post-M052 review at `970252c` accepted the source-truthfulness defect described below.

Pinned authority: I2P Proposal 170 `I2PControl Expansion`, Open, revision `2026-05-20`, plus the read-only i2pd network-error vocabulary adopted by the proposal.

## 1. Objective

Correct exactly these two canonical Proposal 170 selectors:

- `i2p.router.net.error`;
- `i2p.router.net.error.v6`.

Emissary currently has no canonical runtime owner that independently knows the i2pd-style network error state, yet the production wire path maps the internal absence of an error reason (`None`) to numeric code `0`, whose contract meaning is `No error`. This incorrectly turns missing source authority into a positive semantic claim.

M055 must restore truthful behavior by making both selectors explicitly unavailable unless the readiness audit identifies an already-existing production owner that actually distinguishes the required error states. The current repository evidence and M050 closure state that no such owner exists, so demotion is the expected disposition.

Do not reopen or regress the three M050 fields that do have current owners:

- `i2p.router.net.status.v6`;
- `i2p.router.net.testing`;
- `i2p.router.net.testing.v6`.

## 2. Defect and why prior verification missed it

M050 added neutral `NetworkErrorReason` storage, separate v4/v6 atomics, setters, and a numeric mapper. However its own closure records that Emissary has no canonical error owner and therefore production leaves both error reasons as `None`.

The handler currently maps:

- `None -> 0`;
- known reasons to the remaining i2pd numeric codes.

In the adopted reference vocabulary, `0` means an affirmative `No error` state from an actual router-owned error field; it is not an `Unknown/unavailable source` sentinel.

M050 tests populated the new event-handle setters directly and verified the mapper, so they proved that a synthetic value can round-trip. They did not prove that a real production owner ever calls those setters. M052 then exercised the canonical selector and accepted the resulting zero, which again proved wire reachability rather than source authority.

The corrective regression must distinguish `owner reports no error` from `no owner exists`. With the current architecture, direct canonical error requests must fail unavailable rather than serialize `0`.

## 3. Current-state evidence

Current repository evidence establishes:

- v4/v6 firewall/reachability status has real production publication;
- v4/v6 peer-test activity has real production publication;
- no production subsystem has been identified that owns the i2pd-style error reason enum;
- `set_ipv4_network_error()` / `set_ipv6_network_error()` exist in `EventHandle`, but the accepted M050 closure identifies no canonical writer;
- `network_error_code(None)` currently converts source absence to the positive wire code `0`.

M055 must begin with a repository-wide production-writer audit. Test-only writes do not count as a source owner.

## 4. Authorized production path budget

Primary corrective paths:

- `emissary-cli/src/i2pcontrol/router_info_handler.rs`;
- `emissary-cli/src/i2pcontrol/rpc.rs`;
- `emissary-cli/src/i2pcontrol/router_info.rs` / `production.rs` only as needed to remove the unowned error source contract;
- documentation/tests under the existing I2PControl scope.

Core cleanup is authorized only in:

- `emissary-core/src/events.rs`;
- `emissary-core/src/inspection.rs`.

Those two core files may remove error-only scaffolding that has no real production writer/consumer after demotion. Do not touch `transport/**`, `tunnel/**`, `router/**`, NetDB, crypto, I2NP, LeaseSet, AddressBook, proxy/UI, workflows, release, or unrelated code in this corrective pass.

If an actual canonical production error owner is discovered outside these paths, do not wire it under M055. Record the evidence and require a separately authorized owner-specific plan rather than silently expanding this pass.

## 5. Invariants

1. Missing source authority is never serialized as `No error`.
2. Wire code `0` may be emitted only if a canonical runtime owner explicitly reports the semantic state represented by that code.
3. Test-only setters or fake adapters are not production-source evidence.
4. `status.v6` and v4/v6 `testing` remain sourced exactly as accepted by M050; their production paths and wire behavior do not change.
5. Firewall/reachability status must not be reinterpreted as an error reason.
6. Peer-test activity must not be reinterpreted as an error reason.
7. No new reachability probe, error-detection algorithm, ban policy, network task, or transport behavior is added.
8. Remove dead error observation state if and only if the production-writer audit proves it has no retained non-I2PControl use.
9. Proposal 170 numeric mappings remain in I2PControl; core must not acquire wire codes.
10. No upstream interaction or contribution preparation is authorized.

## 6. Work packages

### WP1 — Production-owner audit

Search every production call site of:

- `set_ipv4_network_error`;
- `set_ipv6_network_error`;
- `NetworkErrorReason` construction/publication;
- any adjacent transport/router error state that might appear semantically similar.

Classify each as production owner, test fixture, adapter mapping, or dead scaffolding. Compare only genuine owner semantics to the pinned i2pd error vocabulary.

Expected finding from the current closure is: no production owner exists. If that remains true, proceed with demotion. If a genuine owner unexpectedly exists, stop and record a new owner-specific corrective requirement rather than guessing coverage from partial states.

### WP2 — Demote the two canonical source rows

Change only these two `PROPOSAL_170_CONTRACT` rows from `Available` to `Unavailable` with a precise reason such as `no canonical network-error owner`:

- `P170_NET_ERROR`;
- `P170_NET_ERROR_V6`.

Direct canonical requests must fail through the established source-disposition/unavailable path before partial assembly. Do not insert `0`, `null`, an empty value, or a firewall-derived code.

Remove or make unreachable the canonical direct call to `network_error_code()` for unavailable rows. If the helper becomes unused, delete it rather than retaining dead wire policy.

### WP3 — Remove unowned core scaffolding where safe

If WP1 confirms there are no production error writers and no retained internal consumers:

- remove the v4/v6 network-error atomics from `EventHandle`;
- remove the corresponding setters/accessor plumbing;
- remove `NetworkErrorReason` and `NetworkState.error` only if no other accepted feature requires them;
- update constructors/clones/tests accordingly.

Preserve the already-valid status and testing observations. Do not use this cleanup as justification for broader event/inspection refactoring.

If removing the neutral enum would cause unrelated churn or another legitimate internal consumer exists, it may remain, but the two Proposal 170 rows must still be unavailable until an authoritative owner exists.

### WP4 — Regression and exact error behavior

Add regressions proving:

1. a direct request for `i2p.router.net.error` returns the canonical unavailable JSON-RPC error and no partial result;
2. the same is true for `.error.v6`;
3. requesting either error selector together with valid status/testing selectors still fails the whole canonical request rather than returning partial values;
4. `status.v6` continues to return its accepted numeric mapping;
5. `testing` and `testing.v6` continue to return 0/1 from the existing peer-test source;
6. no fake/test-injected `NetworkErrorReason` can bypass the canonical unavailable disposition;
7. source-count fixtures/documentation reflect the demotion.

The failing-before regression must show that current production can return `0` without any production error owner having published `No error`.

### WP5 — Source matrix and historical disposition

Update source maps, Proposal 170 support docs, roadmap, and registry. Do not rewrite M050's closure as though the defect never occurred. M055 closure must state that M050 remains accepted only for status.v6 and testing v4/v6; its two network-error claims are superseded by this corrective disposition.

M052's prior final matrix is not valid after this finding and remains pending M056 reclosure.

## 7. Failure, cancellation, restart, and contention

Unavailable error selectors fail deterministically before source acquisition; they add no new lock, task, timer, state transition, or restart behavior.

If dead error atomics are removed, restart behavior for retained status/testing stays unchanged. No migration or persistent state exists.

No lock may be introduced across `.await`, network I/O, serialization, or peer-test activity. This pass is a truthfulness correction, not a runtime feature expansion.

## 8. Compatibility, migration, and security

No schema/configuration/persistence/authentication/TLS migration is authorized. Exact selector spelling and integer contract type remain documented, but the current implementation truthfully reports source unavailability.

Historical nested/base network selectors must not be broadened or changed merely to compensate for canonical unavailability.

Removing unowned error scaffolding reduces audited-core surface and is preferred when it can be done without unrelated refactoring. No private data, session handle, socket, channel, or mutable router authority is introduced.

External specification/reference material remains read-only. No upstream issue, PR, review, submission, adoption request, merge request, or maintainer contact is authorized.

## 9. Verification

Run focused owner/source tests first. Then run at minimum:

```bash
cargo check -p emissary-core
cargo test -p emissary-core network_state --no-fail-fast
cargo test -p emissary-core --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol router_info --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m027_literal_fixtures --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test router_info_truthfulness --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test static_guards --no-fail-fast
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-core --all-targets -- -D warnings
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

If a focused test filter no longer exists after removing the error scaffold, run the nearest retained event/inspection suite and record the exact command. Use targeted formatting; do not add CI/fuzz/soak/release infrastructure.

## 10. Documentation and static guards

Document both selectors as unavailable due to absent canonical owners. Add a static/source guard proving the two canonical contract rows cannot be marked available merely because a numeric mapper exists.

Where practical, add a guard that distinguishes test-only setters from production writer evidence. At minimum, closure must include an explicit production call-site audit.

## 11. Acceptance criteria

M055 may close only when:

- both network-error rows are truthfully unavailable unless a separately authorized real owner exists;
- no production path serializes `0` solely because internal error state is `None`/unset;
- direct and combined canonical requests fail without partial results;
- status.v6 and both testing fields remain operational and unchanged;
- the production-writer audit identifies no hidden owner or records a stop condition;
- dead error-only core state is removed when safe, with no broader refactor;
- changed core files are limited to `events.rs` and `inspection.rs`;
- no new probe/error algorithm/network behavior is introduced;
- source documentation and contract counts are reconciled;
- M056, not M055, owns final integrated reclosure.

Expected source accounting after M055 is:

- if M054 restored a correct transit-15s source: 38 available, 1 protocol-permitted neutral, 4 unavailable (`news`, `bannedpeers`, `net.error`, `net.error.v6`);
- if M054 truthfully demoted transit-15s: 37 available, 1 neutral, 5 unavailable.

## 12. Stop conditions

Stop rather than:

- treating firewall state, SSU2 testing state, disconnection, dial failure, or another adjacent signal as an i2pd error code;
- adding a new error-detection subsystem or network probe;
- touching transport state machines merely to manufacture this telemetry;
- retaining `Available` because tests can manually set the error atomics;
- changing M050's valid status/testing fields;
- broadening into news/bans or unrelated Proposal 170 work.

## 13. Closure evidence required

The closure record must include the production-writer audit, failing-before zero-without-owner proof, direct/combined unavailable regression, changed-path/core-scaffold audit, retained status/testing evidence, exact source-count reconciliation, no-feature verification, security/compatibility review, and internal-only attestation.

M056 remains blocked until M054 and M055 both have accepted closure dispositions.