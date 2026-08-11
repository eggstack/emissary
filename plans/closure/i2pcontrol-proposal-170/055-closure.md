# M055 Closure Record — M050 Network-Error Truthfulness Corrective

Status: closed

Reviewed plan: `plans/implementation/i2pcontrol-proposal-170/055-m050-network-error-truthfulness-corrective.md`

Implementation commit: recorded in the final commit history for this closure

Closure date: 2026-08-11

## Disposition

M055 is accepted as a bounded corrective pass. Both canonical Proposal 170
selectors `i2p.router.net.error` and `i2p.router.net.error.v6` are explicitly
unavailable because Emissary has no canonical production owner for the adopted
i2pd-style network-error reason state. The prior `None -> 0` mapping, which
claimed `No error` from source absence, is removed.

M050 remains accepted only for:

- `i2p.router.net.status.v6`;
- `i2p.router.net.testing`;
- `i2p.router.net.testing.v6`.

Its two network-error claims are superseded by this corrective disposition.
M052's historical integrated matrix remains invalidated and M056 is now ready
to perform the independent final reclosure.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Audit every production writer before demotion | Repository-wide search found only the former `EventHandle` setters and their M050 unit fixture; no production call site publishes a `NetworkErrorReason` | pass |
| Do not infer errors from adjacent state | `FirewallStatus`, reachability status, and peer-test activity remain used only for status/testing; no error code is derived from them | pass |
| Demote both canonical rows | `rpc.rs::PROPOSAL_170_CONTRACT` marks both rows `Unavailable` with owner `network-error` and reason `no canonical network-error owner` | pass |
| Fail before source acquisition and assembly | Existing canonical source-disposition preflight rejects both rows before network snapshot acquisition; direct and combined regressions assert `-32603`, null result, and the sanitized reason | pass |
| Never serialize unset state as code 0 | `network_error_code` and all error serialization calls are removed; static guards reject the old mapper and error source scaffolding | pass |
| Preserve status.v6 and testing v4/v6 | The retained wire fixture and full RouterInfo suite continue to assert status `1` and testing `1/0`; production `NetworkState` retains only status/testing | pass |
| Remove dead core scaffolding where safe | `NetworkErrorReason`, error fields, v4/v6 error atomics, setters, atomic conversion helpers, and related fixture writes were removed from `inspection.rs`/`events.rs`; no other core paths consumed them | pass |
| Keep the Proposal 170 boundary in I2PControl | Numeric mapping policy remains absent from core; contract ownership and unavailable response policy remain in `rpc.rs`/`router_info_handler.rs` | pass |
| Reconcile source accounting | Contract, literal fixture, conformance manifest, source map, support docs, and RouterInfo docs now report 43 total / 37 available / 1 neutral / 5 unavailable | pass |
| Keep future planning state correct | M055 is closed; M056 is registered `ready`; M051 remains blocked by absent news/ban owners | pass |

## Production-writer audit

The audit covered all repository references to:

- `set_ipv4_network_error`;
- `set_ipv6_network_error`;
- `NetworkErrorReason` construction/publication;
- `ipv4_network_state` and `ipv6_network_state` consumers;
- the adjacent firewall/status/testing observations.

Before this change, the only writes were direct calls in the `events.rs` unit
fixture. There were no production calls from transport, router, SSU2, NetDB,
or any other runtime subsystem. The former I2PControl adapter copied the unset
state into a DTO and the handler converted `None` to wire code `0`; this was an
adapter path, not evidence of an owner. No genuine canonical owner was found,
so the plan's stop condition did not apply and demotion proceeded.

The failing-before proof is the historical pre-change handler branch:
`network_error_code(None)` returned `0`, while the production writer audit
found no writer capable of reporting the semantic `No error` state. The new
static guard and unavailable-request regressions preserve that distinction in
executable evidence: a fake/test adapter cannot make either selector
requestable, and no mapper remains that can turn absence into `0`.

## Core-scaffold and changed-path audit

Core changes are limited to:

- `emissary-core/src/events.rs`;
- `emissary-core/src/inspection.rs`.

I2PControl changes are limited to the network snapshot/contract/handler
composition seams and their tests. No transport, tunnel, router, NetDB,
crypto, I2NP, LeaseSet, AddressBook, proxy/UI, workflow, release, or external
integration code was changed. No probe, error-detection algorithm, network
task, timer, persistence, lock, or lifecycle behavior was added.

## Regression evidence

The focused regression `network_errors_are_unavailable_without_partial_results`
covers direct v4, direct v6, and a combined request containing both error rows
plus valid status/testing selectors. Every case fails as an unavailable internal
JSON-RPC error with a null result. The live TLS child-process test covers both
selectors through the authenticated production path and asserts the same
disposition.

The retained network-state fixture confirms:

- `i2p.router.net.status.v6` remains `1` for firewalled status;
- `i2p.router.net.testing` remains `1` when the v4 test is active;
- `i2p.router.net.testing.v6` remains `0` when the v6 test is inactive.

The contract and static guards verify that the two error rows cannot be made
available merely because a serializer or numeric fixture exists, and that the
former test-only error setters and enum do not return as hidden source
authority.

## Verification

Successful checks:

- `cargo check -p emissary-core`.
- `cargo test -p emissary-core network_state --no-fail-fast` — 1 passed.
- `cargo test -p emissary-core --no-fail-fast` — 1062 passed, 2 ignored.
- `cargo check -p emissary-cli --no-default-features`.
- `cargo test -p emissary-cli --no-default-features --no-fail-fast` — 56 passed.
- `cargo check -p emissary-cli --no-default-features --features i2pcontrol`.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast` — 1372 passed.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol router_info --no-fail-fast` — 135 passed.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest --no-fail-fast` — 58 passed.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m027_literal_fixtures --no-fail-fast` — 7 passed.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test router_info_truthfulness --no-fail-fast` — 36 passed.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test static_guards --no-fail-fast` — 40 passed.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime live_runtime_interoperability --no-fail-fast` — 1 passed.
- `cargo clippy -p emissary-core --all-targets -- -D warnings`.
- `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings`.
- `git diff --check`.
- targeted `rustfmt` over every changed Rust file.

`cargo fmt --all -- --check` remains blocked by extensive pre-existing
workspace-wide formatting differences in untouched files under the active
stable toolchain. No unrelated formatting churn was absorbed. This is the
same repository baseline qualification recorded by M054.

## Compatibility, security, and operational review

The JSON-RPC method, selector spelling, declared integer contract type, direct
presence behavior, authentication boundary, and error code remain unchanged.
Only the source disposition changes from an invalid available value to the
established unavailable path. No migration or persisted-state change exists.

Unavailable preflight is deterministic and occurs before source acquisition,
serialization, or partial result assembly. Removing unused atomics and DTO
fields reduces audited state and introduces no new lock, await point, task,
socket, channel, private material, or router authority. Default/no-I2PControl
behavior remains green.

All work and references were internal/read-only with respect to external
specifications. No upstream issue, pull request, review, submission, or
maintainer contact was made.

## Planning handoff

- M055: `closed`.
- M056: `ready`, because M054 and M055 now both have accepted closure records.
- M051: remains `blocked with accepted semantic limitation`; no news or ban
  owner was created or inferred by this pass.
- M052: remains historical/invalidation evidence until M056 performs its
  production-free integrated reclosure.

No unresolved high- or medium-severity finding remains within M055's bounded
scope. The two network-error fields are an explicit accepted limitation, not a
claim of runtime capability.
