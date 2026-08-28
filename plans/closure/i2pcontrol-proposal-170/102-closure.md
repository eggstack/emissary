# M102 Closure — RouterInfo Network-Error Owner Completion

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/102-routerinfo-network-error-owner-completion.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Repository baseline reviewed: `33b2cf5` (pre-M102 implementation baseline)

Implementation commits:

- `19bdf6cee79466260b3d05bcf8e17a3ed51a6c11` — add bounded neutral IPv4/IPv6 network-error observation, I2PControl wire mapping, tests, and active-plan reconciliation

Review date: 2026-08-28.

## 1. Executive finding

M102 closes as implemented. Both Proposal 170 network-error rows now have an
explicit, family-independent runtime owner: `TransportManager::on_firewall_status`
publishes the existing SSU2 reachability result into independent IPv4 and IPv6
inspection state. `Ok` is represented as `NoError` (wire code `0`) only after
an explicit successful evaluation; `SymmetricNat` is represented as the
observed symmetric-NAT condition (wire code `3`). Uninitialized, `Firewalled`,
and `Unknown` states remain unavailable and never become a fabricated zero.

The core state is protocol-agnostic. Proposal 170 names, serializers, and
integer mapping remain in `emissary-cli/src/i2pcontrol/`. No transport decision,
router algorithm, persistent schema, or external protocol behavior changed.
The authoritative production matrix is now 43 total: 41 available, 1
protocol-permitted neutral, and 1 unavailable (`i2p.router.netdb.bannedpeers`,
reserved for M103).

The adopted i2pd reference vocabulary was checked read-only against
`RouterContext.h`: `None = 0`, `ClockSkew = 1`, `Offline = 2`,
`SymmetricNAT = 3`, `FullConeNAT = 4`, and `NoDescriptors = 5`.
Only `None` and `SymmetricNAT` are representable from the existing Emissary
owner without inference; the other reasons have no canonical family-specific
Emissary writer and were intentionally not invented.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| M095 hard gate reconciled before code | M102 plan §17 and the field-by-field writer table below | pass | The readiness audit completed the writer/clear/source table that was missing from the candidate inventory. |
| Explicit IPv4 state | `EventHandle::set_ipv4_network_error`; `TransportManager::on_firewall_status`; core unit test | pass | IPv4 is stored and read independently. |
| Explicit IPv6 state | `EventHandle::set_ipv6_network_error`; `TransportManager::on_firewall_status`; core unit test | pass | IPv6 is stored and read independently. |
| Exact neutral semantics | `NetworkErrorReason::{NoError,SymmetricNat}` | pass | Only observable neutral conditions are represented. |
| Exact adopted wire mapping | `network_error_code` and handler unit test | pass | `NoError -> 0`; `SymmetricNat -> 3`; absent state errors. |
| No fabricated zero | uninitialized handler test and `Option<NetworkErrorReason>` state | pass | `None` returns sanitized unavailable `-32603` behavior. |
| Direct presence/source semantics | contract rows and RouterInfo handler tests | pass | Both rows are requestable only through the canonical direct contract. |
| Passive observation only | `on_firewall_status` setter calls and transport regression test | pass | State is not consumed by routing, firewall, or transport decisions. |
| No protocol logic in core | static guards scan core sources for Proposal 170, JSON-RPC, and selector names | pass | Core remains administrative-protocol agnostic. |
| No persistent migration | code and compatibility review | pass | State is bounded process memory and resets on construction/restart. |
| Containment budget | M061/M062 suites and M102 path guard | pass | Changed core seams are the existing inspection/event/transport owner paths. |
| Remaining-row accounting | M095 matrix, conformance manifest, literal fixtures, and active docs | pass | Counts agree at 41/1/1; M103 owns the sole unavailable row. |

## 3. Production implementation evidence

The canonical writer/clear table is:

| Family | Reason | Set event | Clear event | Owner/source |
|---|---|---|---|---|
| v4 | `NoError` | SSU2 firewall-status reports `Ok` | SSU2 firewall-status reports `Firewalled` or `Unknown` | `TransportManager::on_firewall_status` |
| v4 | `SymmetricNat` | SSU2 firewall-status reports `SymmetricNat` | SSU2 firewall-status reports `Firewalled`, `Unknown`, or `Ok` | `TransportManager::on_firewall_status` |
| v6 | `NoError` | SSU2 firewall-status reports `Ok` | SSU2 firewall-status reports `Firewalled` or `Unknown` | `TransportManager::on_firewall_status` |
| v6 | `SymmetricNat` | SSU2 firewall-status reports `SymmetricNat` | SSU2 firewall-status reports `Firewalled`, `Unknown`, or `Ok` | `TransportManager::on_firewall_status` |

The state is carried through `NetworkState.error` and the production
`NetworkSnapshot`. I2PControl reads the snapshot once per request and maps
the typed reason to the adopted integer. Reads do not mutate state, and v4/v6
fail independently when one family lacks a valid evaluation.

Read-only external evidence was the official [Proposal 170 page](https://i2p.net/en/proposals/170-i2pcontrol-expansion/)
and the Debian mirror of [i2pd `RouterContext.h`](https://sources.debian.org/src/i2pd/2.45.1-1/libi2pd/RouterContext.h).
Those sources were used only to reconcile vocabulary and numeric meaning; no
upstream or third-party repository was modified.

## 4. Verification executed

### Commands run

```text
cargo check -p emissary-core
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-core --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest --test m027_literal_fixtures --test m062_dependency_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
cargo fmt --all -- --check
```

### Results

| Command/result | Outcome |
|---|---|
| Core checks/tests | pass; 1,064 passed, 2 ignored |
| Feature-gated CLI checks/tests | pass; 1,737 passed |
| Conformance/literal/M062 focused suites | pass; 84 passed |
| M061/M062 containment suites | pass; 26 passed |
| Clippy with `-D warnings` | pass; no issues |
| `git diff --check` | pass |
| `cargo fmt --all -- --check` | qualified failure; existing unrelated files still differ under stable `rustfmt 1.9.0`, and the repository config requests nightly-only options. Newly changed Rust sections were formatted and no unrelated formatter churn was accepted. |
| M063 feature-reachability target | not run: this checkout has no `m063_feature_reachability` Cargo test target. Existing M063 semantic guards and feature-disabled checks remain part of the repository evidence and passed where available. |

The initial full CLI run caught stale 39/3 count expectations and an overly
specific uninitialized fake-source assertion; those test-only defects were
corrected and the complete rerun passed. No production test failure remains.

## 5. Invariant review

- Network-error values are observations, not inferred protocol classifications.
- `Option<NetworkErrorReason>` distinguishes no evaluation from `NoError`.
- IPv4 and IPv6 state have separate storage, writers, clearing, and reads.
- Private atomic tags are storage discriminants only and do not mirror external
  protocol numbering.
- Core has no Proposal 170 selector, JSON-RPC, or wire-code dependency.
- Network-error observation does not alter transport selection, firewall
  handling, routing, or peer admission.
- The bounded event snapshot is reset on process restart; no stale persisted
  reason can be exposed.
- Unavailable source state fails before result assembly and cannot produce a
  partial response.
- The remaining banned-peer row is not opportunistically reclassified by M102.

## 6. Failure and recovery review

The state machine is bounded and overwrite-based: each family has one latest
value, and a later `Firewalled`/`Unknown` result clears it. Repeated status
events do not allocate, queue, or create duplicate tasks. Concurrent reads
observe a coherent atomic value for each family and never mutate it. If a
request asks for both families, either family can independently produce an
unavailable result before assembly; no partial result is returned.

Malformed or unauthorized requests continue to use the existing RouterInfo
parser, authentication, direct-selector, and sanitized JSON-RPC error paths.
There is no new input-controlled allocation, network operation, persistence,
retry loop, cancellation race, or cleanup obligation.

## 7. Migration and compatibility review

No schema, configuration, lockfile, or wire migration is required. Existing
clients requesting the two selectors gain integer values after a valid family
evaluation. Before evaluation, and for `Firewalled`/`Unknown`, the established
unavailable error remains truthful. Legacy nested RouterInfo compatibility
semantics are unchanged. Rollback is limited to reverting the implementation
commit; there is no persistent state to migrate or repair.

## 8. Security review

Authentication and authorization remain in the existing I2PControl request
path. The new values contain no secrets and are read-only. No filesystem,
privilege, destination, socket, or external-fetch behavior was added. Error
messages remain sanitized. The state is bounded by two atomics and does not
create a denial-of-service surface.

## 9. Documentation and operations

Updated architecture/support evidence includes:

- `docs/i2pcontrol/README.md`
- `docs/i2pcontrol/proposal-170-conformance.md`
- `docs/i2pcontrol/proposal-170-support.md`
- `docs/i2pcontrol/router-info-source-map.md`
- `docs/i2pcontrol/router-info.md`
- M095 matrix, implementation index, registry, and full-support roadmap

The static contract, literal availability, production adapter, source-boundary,
and dependency-containment tests now guard the 41/1/1 state and M102 path
budget. Operators should treat an uninitialized or firewalled family as
temporarily unavailable rather than as a healthy network result.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | The requested `m063_feature_reachability` Cargo test target is absent from this checkout. | The named historical command cannot be executed verbatim; existing M063 semantic guards remain available. | Preserve the limitation in closure records; do not create an unrelated replacement under M102. |
| low | Repository-wide stable `cargo fmt --check` reports pre-existing formatting drift and ignored nightly-only configuration. | Full formatting gate is not green independently of this change. | Resolve in a dedicated formatting/toolchain plan if desired; no M102 correctness impact. |

No critical, high, or medium correctness, security, compatibility, or
containment finding remains.

## 11. Roadmap disposition

M102 is closed and M103 is unblocked: it is the next ready/executable handoff
for the sole remaining unavailable `i2p.router.netdb.bannedpeers` row. M098
and M099 remain blocked on M097's named Yosemite/SAM and key-lifecycle
primitives. M104 remains blocked on M097-M103 and still requires integrated
public/reference-router interoperability and final security/containment
reclosure. No other future plan becomes unblocked from M102.

## 12. Registry updates

Completed in the implementation commit:

- marked the M102 plan closed and linked this closure record;
- marked M102 closed and M103 ready in `plans/registry.md`;
- updated the full-support implementation index and subsystem roadmap;
- reconciled the authoritative M095 matrix to 41 available / 1 neutral / 1 unavailable;
- updated active RouterInfo/support/conformance documentation;
- added the M102-specific containment authorization to the existing M062 guard.

Historical M055, M056, and M101 closure records were preserved unchanged.

**Disposition: closed.**
