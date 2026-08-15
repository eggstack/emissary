# M072 Closure — Proposal 170 Tunnel Runtime Completion Reclosure

Status: closed after M073 corrective pass

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/072-tunnel-runtime-completion-reclosure.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`

Planning production baseline:

- `a1296b018ce98d26a019bd5064dff9f4b47e0ad6`

## 1. Executive finding

M072 independently reclosed the twelve-type production composition, inherited
family security evidence, lifecycle ownership, containment, feature isolation,
and support claims. The specialized M066–M071 families are operational within
their declared capability sets. M072 cannot be accepted as runtime-completion
closed because the pre-existing generic `client` and `server` backends have a
material option-truthfulness defect: runtime-relevant typed and raw options can
be persisted and accepted at start without being applied or rejected.

The exact integrated matrix is retained in
`plans/closure/i2pcontrol-proposal-170/072-option-capability-matrix.toml`.
M073 is created as the bounded corrective successor; it must repair the generic
option boundary before this roadmap can close.

## 2. Implementation and closure evidence

The reclosure covers the accepted implementation sequence:

| Milestone | Implementation/closure evidence |
|---|---|
| M064 | `2be4518` implementation; `a310927` closure/advance |
| M065 | `6c55748`; closure in `plans/closure/i2pcontrol-proposal-170/065-closure.md` |
| M066 | `7fd7408`; closure in `plans/closure/i2pcontrol-proposal-170/066-closure.md` |
| M067 | `4512966`; `b51999e` closure |
| M068 | `cb76892`; `d36d637` closure |
| M069 | `3a9a4a8`; `018ddb7` closure/advance |
| M070 | `3c07797`; closure in `plans/closure/i2pcontrol-proposal-170/070-closure.md` |
| M071 | `55cb30a`; closure in `plans/closure/i2pcontrol-proposal-170/071-closure.md` |

These are internal repository commits only; no upstream pull request or
external review artifact exists.

## 3. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Exactly twelve pinned types/actions and unchanged schema | `domain/tunnel.rs`, registry compile-time guards, package tests | pass |
| All twelve production types are real | composed production registry test; `production.rs` composition | pass |
| Default/test registry remains intentionally unsupported and resource-free | default-registry tests; M061 containment tests | pass |
| Specialized backends validate options before allocation | M066–M071 closures and package tests | pass |
| Generic `client` applies/rejects every runtime-relevant option | M073 closure; `client.rs`, `options.rs` | pass |
| Generic `server` applies/rejects every runtime-relevant option | M073 closure; `server.rs`, `options.rs` | pass |
| Lifecycle generation, cancellation, restart, and server identity | M065–M071 closures; backend lifecycle tests | pass for reviewed paths |
| HTTP inbound/outbound filtering and framing safety | M067/M068/M070 closures and filter tests | pass |
| IRC common filter, registration identity, DCC/WEBIRC fail-closed | M066/M069 closures and filter tests | pass |
| SOCKS/CONNECT routing, auth, and no-local-DNS policy | M068/M069 closures and routing tests | pass |
| Streamr bounds/control/refresh and restart state | M071 closure and Streamr tests | pass |
| Secret redaction and path confinement | server store tests, M065–M071 closure evidence | pass |
| Startup ownership and heterogeneous `All` review | `production.rs`, lifecycle tests, inherited closure evidence | pass |
| M061 source containment | `m061_containment` | pass |
| M062/M063 dependency containment | `m062_dependency_containment` | pass |
| M064 remains the only planned core correction | baseline diff and containment review | pass |
| RouterInfo remains 37/1/5; M051 remains separate | support docs and registry | pass |
| Documentation truthfully distinguishes partial Proposal 170 support | docs review | pass after status updates in this change |
| No high/medium finding remains | M073 closure and M074 closure | pass for M072 scope |

## 4. Confirmed unclosed finding

`CLIENT_OPTIONS` declares `AccessList`, `AllowPlaintext`, I2CP options, and
custom options accepted, but `ClientTunnelRuntimeConfig` consumes only the
destination, target port, listen interface, and listen port. The generic client
also has no raw-option allowlist. A runtime-relevant raw key can therefore be
stored and ignored.

`SERVER_OPTIONS` declares `IsPrivate`, `HashCash`, `SignatureType`, `Consumer`,
I2CP options, and custom options accepted, but the runtime consumes only the
loopback target/port and `i2cp.leaseSetEncType`. The generic server also has no
raw-option allowlist. The remaining accepted fields can be stored and ignored.

This is a medium-severity correctness/security finding because callers can
believe access, privacy, identity, or tunnel-shaping controls are active when
they are not. It is not absorbed into M072 because changing the acceptance
boundary changes existing option semantics and is outside M072's permitted
small direct-fix budget.

Corrective plan: `plans/implementation/i2pcontrol-proposal-170/073-generic-tunnel-option-truthfulness-corrective.md`.

## 5. Verification executed

Passed:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

The package test run passed 1,520 tests across 24 suites after replacing two
redundant registry assertions with one exhaustive composed-registry test. M061
and M062/M063
containment suites are included in that result and were also reviewed directly.

Known inherited/non-M072 limitations:

- `cargo check -p emissary-core --no-default-features` fails on existing
  feature-disabled `RwLock` imports in unrelated core modules;
- stable `cargo fmt --all -- --check` fails on repository-wide nightly-only
  rustfmt configuration and existing formatting drift;
- no public-network tunnel formation or hosted CI/release expansion was added.

## 6. Invariant and containment review

The production path uses the composed registry with a fixed backend-owned
`server-destinations/` store, and all twelve backends inspect stopped rather
than unsupported in that composition. The default registry remains a
dependency-light test contract. Specialized HTTP, IRC, SOCKS/CONNECT, and
Streamr paths retain their accepted ownership, bounds, cancellation, target
confinement, and filter guarantees. No new `emissary-core/**` runtime path was
introduced; M064 remains the only planned core correction.

The remaining failure is specifically the generic option matrix, not a missing
family backend, filter bypass, lifecycle orphan, secret leak, dependency
contamination, RouterInfo source claim, or public protocol expansion.

## 7. Documentation and support disposition

The support and backend documentation is updated to state that all twelve
families are real in production composition while the overall Proposal 170
status remains partial because RouterInfo has five unavailable rows and other
base-method/AddressBook limitations remain. DCC, WEBIRC, SOCKS BIND/UDP,
unsupported TLS/custom/I2CP fields, outproxy/DNS/LAN policy, server filtering,
and Streamr bounds remain explicitly described.

The generic client/server option limitation is called out as a corrective
blocker rather than represented as complete support.

## 8. Future-plan unblock review

- M073 is closed as the bounded corrective successor for the generic option
  finding.
- M074 is closed as the next security-hardening milestone and owns the shared
  accepted-server admission boundary.
- M051 remains independently blocked: no substantive router-news or banned-peer
  owners exist, and M072 creates no such owner.
- Containment M061 and dependency M062/M063 remain closed authorities; no
  corrective re-opening is needed.

## 9. Internal-only attestation

External specifications and reference material were accessed read-only. No
upstream repository, issue, pull request, review, merge request, maintainer
channel, branch, tag, release, contribution package, or submission artifact was
created or prepared. Repository writes remained within the authorized internal
`eggstack/emissary` repository. The explicit user directive authorizes the
eventual commit and push of this internal branch only.

## 10. Final disposition

**Closed.** M072 is formally accepted after M073 proved that generic `client`
and `server` runtime-relevant options are applied or rejected before
allocation. The later M074-M079 security sequence remains a separate corrective
workstream governed by its own closures.
