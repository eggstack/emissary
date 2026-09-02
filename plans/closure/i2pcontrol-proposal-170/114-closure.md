# M114 Closure — Full Proposal 170 Live Interoperability and Final Reclosure

Status: **closed as blocked**

Plan: `plans/implementation/i2pcontrol-proposal-170/114-full-proposal-170-live-interoperability-and-final-reclosure.md`

Reviewed implementation head: `47e4d47838b187735625329e400e02af662d0252`

Closure date: 2026-09-02

Pinned authority: I2P Proposal 170 revision `2026-05-20`, status Open.

## Disposition

M114 completed its independent final reclosure review and is closed as blocked. The
current implementation is a truthful partial Proposal 170 implementation, not a full
support claim. The plan's hard readiness gate remains false because 70 applicable
TunnelManager option/type cells are still `blocked_primitive`, and the required
reference-router and public/reseeded-network evidence could not be obtained in this
environment.

No production feature was added under M114. This is required by the plan: M114 is a
reclosure milestone and must not implement a missing primitive owned by M111–M113 or
weaken a security boundary to make an evidence gate pass.

## Readiness gate

| Gate | Current-head evidence | Result |
|---|---|---|
| M109 closed | `plans/closure/i2pcontrol-proposal-170/109-closure.md` | pass |
| M110 closed/corrected | M110 closure plus M116 corrective closure | pass |
| M111 has no unresolved applicable cells | M111 closure retains four `UseSSL` cells as blocked | **fail** |
| M112 has no unresolved applicable cells | M112 closure retains 45 residual cells as blocked | **fail** |
| M113 has no unresolved applicable cells | M113 closure retains 21 presentation/routing/LeaseSet cells as blocked | **fail** |
| M095 has zero applicable residuals | `312 apply / 70 blocked_primitive / 458 not_applicable` | **fail** |
| M105 is reconciled to zero applicable residuals | Post-M112/M113 reconciliation retains the same 70 cells | **fail** |
| No high/medium Proposal-scoped security corrective is open | M093 and later closure reviews; no new finding in this pass | pass |

The failed gates are substantive blockers, not test or documentation failures. M111,
M112, and M113 are correctly recorded as closed as blocked; their closure does not
satisfy M114's stricter zero-residual requirement.

## Reviewed head and artifact hashes

The reviewed head is the M113 closure head. M117 and M118 are already included in
its history and do not promote any residual matrix cell.

The relevant resolved dependency/runtime versions at that head are Rust/Cargo
`1.97.1`, `axum 0.8.4`, `tokio 1.48.0`, registry `yosemite 0.7.0`, and the optional
I2PControl-only `yosemite-i2pcontrol` package `0.7.0` at git revision
`8026f5b424fc178d683e63555335f8b33e0aba04`. The disabled dependency tree contains
only registry Yosemite; the enabled tree contains both the registry package and the
exact fork revision.

| Artifact | SHA-256 | Current interpretation |
|---|---|---|
| `095-full-support-matrix.toml` | `9fea6844e0b7e28959e1169491d100ce2f81124fff790f6c10882b765b41eea9` | 840 cells: 312 apply, 70 blocked, 458 not applicable |
| `105-residual-option-audit.toml` | `4817caa1673711bac6d7867319b1e5cf96daeca47acff19a3f605bda521b489b` | historical audit reconciled by its post-M111/M112/M113 fields to the current 70 residual cells |

The exact contract inventory remains mechanically stable:

- 43 Proposal 170 RouterInfo additions: 42 available, 1 protocol-permitted neutral,
  0 unavailable;
- four AddressBook books, canonical modes, subscriptions, and all 13 SetConfig keys;
- 12 canonical tunnel types and seven canonical lowercase actions;
- six ClientServicesInfo selectors;
- 840 option/type cells with no `planned_apply`, `unknown`, `unsupported`, or
  `accept_inert` dispositions.

The last line is not equivalent to full support: the 70 applicable blocked cells are
truthful fail-before-allocation or unavailable-primitive dispositions, so the M095
zero-residual gate still fails.

## Requirement-to-evidence matrix

| M114 work package | Evidence | Disposition |
|---|---|---|
| WP1 canonical inventory | M095/M105 machine-readable artifacts; conformance and matrix tests | passed for inventory; failed zero-residual criterion |
| WP2 wire/golden conformance | 174 focused conformance, golden, literal-fixture, and service-selector tests; 1,841 feature-enabled CLI tests | passed for implemented/claimed surface; blocked options remain truthful |
| WP3 local production composition | `i2pcontrol_live_runtime`: managed TLS/auth, AddressBook mutation/restart/path rejection, request-time RouterInfo, real ClientServicesInfo, mixed startup/control-plane lifecycle, bind-failure rollback, edit/restart, durable recovery, bounded shutdown | passed for available local evidence |
| WP4 twelve-family traffic evidence | No current-head direct I2P data-plane traffic test was available without a reseeded router/SAM peer environment | blocked |
| WP5 reference-router interoperability | No disposable Java I2P/i2pd counterpart could run in this environment | blocked by environment |
| WP6 public/reseeded truthfulness | No safe reseeded/public Emissary run was available; local fixture records unavailable network sources truthfully | blocked by environment |
| WP7 independent security reclosure | M093 authority plus M107/M108 and M109–M118 closure reviews; current feature-enabled tests and clippy pass | no new high/medium finding; residual capability blockers remain |
| WP8 containment/diff audit | M113 head contains no M114 production delta; this closure adds only planning bookkeeping and the M062 closure-path guard | passed |

### Twelve-family operational evidence

M114 does not infer traffic interoperability from registration. The bounded live fixture
exercises real production composition and client/server lifecycle failure paths, but its
SAM-dependent tunnel formation is unavailable without a reseeded peer set. Therefore no
family receives a false traffic certification here:

| Canonical type | Direct current-head traffic evidence |
|---|---|
| `client` | unavailable; local fixture covers listener lifecycle and bounded formation failure |
| `httpclient` | unavailable |
| `ircclient` | unavailable |
| `socks` | unavailable |
| `socksirc` | unavailable |
| `connectclient` | unavailable |
| `streamrclient` | unavailable |
| `server` | unavailable; local fixture covers accepted-server lifecycle failure |
| `httpserver` | unavailable |
| `httpbidirserver` | unavailable |
| `ircserver` | unavailable |
| `streamrserver` | unavailable |

The type registry and backend ownership tests still prove the exact 12-type inventory;
they are not substituted for the missing traffic evidence.

## Local runtime, recovery, and contention evidence

The current-head live fixture passed through a real feature-enabled HTTPS child process
and verified:

- API 1 authentication, wrong-password rejection, token placement, notifications,
  JSON-RPC IDs, and explicit-null behavior;
- AddressBook private mutation, lookup, deletion, persistence across restart, and
  confinement rejection for an unsafe SetConfig path;
- request-time RouterInfo values, including available sources, neutral/unavailable
  values, and multi-selector composition;
- actual ClientServicesInfo state, including the unavailable BOB service;
- startup and control-plane tunnel inventory/action ownership;
- failed client bind followed by edit/restart without resetting the process;
- failed local SAM-dependent client/server formation without fabricated success;
- durable tunnel/address-book recovery after process restart;
- password absence from child diagnostics and bounded process cleanup.

The M109–M118 closure records provide the focused evidence for the remaining lifecycle
properties: atomic state snapshots, failed create/edit/start rollback, generation-local
cancellation, shared-session teardown/refcounts, Streamr producer isolation, secret
redaction, managed-TLS fail-closed startup, and neutral SAM tunnel-pool variance/backup
shutdown. No M114 code change invalidated those reviews.

## Reference and public-network limitation

The environment was checked before closing the gate:

- Docker `29.7.2` is installed, but access to `/var/run/docker.sock` is denied;
- `i2pd` is not installed;
- Java is available only as OpenJDK `25.0.4`; no Java I2P router/reference runtime is
  installed;
- no reseeded/public Emissary network configuration was available for a bounded safe
  run.

Consequently M114 records the exact limitation and does not claim Java I2P, i2pd, SAM
streaming/datagram, encrypted LeaseSet, HTTP/SOCKS/IRC, or Streamr interoperability.
No external repository, reference source tree, issue, pull request, configuration, or
maintainer channel was modified.

## Security, compatibility, and containment review

- API 1-only authentication, HTTPS-only listener behavior, managed TLS, bounded token
  and throttle state, and secret-safe diagnostics remain covered by M107/M108 and the
  current tests.
- RouterInfo and ClientServicesInfo values are sourced from real owners or return the
  documented neutral/unavailable result; no fabricated network or service state was
  introduced.
- AddressBook runtime precedence and confined persistence remain owned by one enabled
  runtime authority.
- Server target routing remains literal-loopback confined. There is no direct I2P to
  clearnet DNS fallback, no request-selected local-target expansion, and no LeaseSet
  security downgrade.
- Trusted remote identity, bounded server admission, HTTP/IRC filtering, Streamr
  subscriber/payload/fanout limits, shared-session isolation, key persistence, and
  generation-local lifecycle ownership remain under their prior closure authorities.
- The ordinary/default feature path remains unaffected; M117's exact optional Yosemite
  fork alias is feature-owned and M118's neutral core changes are already authorized.
- Changed paths in this closure are limited to the M114 plan, this closure record, the
  Proposal 170 planning README, registry, roadmap, and the M062 allowlist entry for
  this closure record. There is no unexplained core/util/dependency/frontend/workflow
  production expansion.

No new high- or medium-severity Proposal 170 security/correctness finding was found.
The stable/nightly rustfmt configuration mismatch remains a pre-existing low-severity
tooling issue; no formatter churn was retained.

## Verification record

All commands were run from the repository root on 2026-09-02.

| Command | Outcome |
|---|---|
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | PASS |
| `cargo check` | PASS |
| `cargo test -p emissary-core` | PASS — 1,069 passed, 2 ignored, 5 suites, 122.89s |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | PASS — 1,841 passed, 26 suites |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment` | PASS — 30 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit` | PASS — 3 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture` | PASS — 1 test, 20.89s |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest --test golden_fixtures --test m027_literal_fixtures --test client_services_live --test static_guards --no-fail-fast` | PASS — 174 tests |
| `cargo tree -p emissary-cli --no-default-features --edges normal` | PASS — registry Yosemite only; no fork URL |
| `cargo tree -p emissary-cli --no-default-features --features i2pcontrol --edges normal` | PASS — registry Yosemite plus exact fork revision |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | PASS — no issues |
| `cargo fmt --all -- --check` | FAIL — pre-existing stable/nightly configuration mismatch reports broad unrelated churn; none retained |
| `git diff --check` | PASS |

The M095/M105 tests also confirm there are no planned, unknown, unsupported, or
accept-inert cells. This is a truthful partial-support result, not a passing M114
readiness gate.

## Future-plan readiness audit

The dependency/status audit found no plan that can be promoted to `ready` by this
closure:

- M111 is already closed, but its four `UseSSL` cells remain blocked because the
  Proposal's local application/session TLS effect is not Yosemite SAM-control TLS.
- M112 is closed as blocked with 45 residual proxy/plugin/profile/reduction/Streamr
  lifecycle cells. It needs a separately accepted owner or architecture decision;
  M114 does not authorize one.
- M113 is closed as blocked with 21 residual presentation/routing/LeaseSet cells. It
  needs an accepted, fail-closed LeaseSet/auth primitive or a separately accepted
  bounded presentation/routing architecture; M114 does not authorize one.
- M115–M118 are already closed. M117/M118 prerequisites do not alter the matrix.
- No subsequent Emissary implementation plan exists that is dependency-ready. The
  registry therefore records M114 as closed as blocked and no current handoff. A new
  numbered corrective must be registered only after its exact primitive, security
  boundary, and ownership are accepted; then a fresh final reclosure can be planned.

This closure does not change the official status from **partial Proposal 170 support**
and does not state full support against the pinned revision.

## Internal-only attestation

All external specifications, reference implementations, and dependency sources were
treated as read-only evidence. No upstream or third-party repository, issue, pull
request, review, maintainer channel, release, branch, tag, or external configuration
was mutated. The resulting planning and test bookkeeping is internal to
`eggstack/emissary`.
