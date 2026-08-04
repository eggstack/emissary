# I2PControl Proposal 170 Milestone M029 — Closure

Status: partial Proposal 170 support

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/029-in-scope-conformance-reclosure.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

## 1. Executive finding

M029 is closed as the controlling final-head review for the authorized
internal scope. Every implemented and claimed dimension passes independent
review and local evidence. The truthful final disposition is `partial Proposal
170 support`: 26 of the 43 Proposal 170 RouterInfo additions remain explicitly
unavailable, and the twelve missing tunnel data planes remain explicit,
resource-free unsupported runtimes. No high- or medium-severity correctness,
security, compatibility, scope, or claim defect remains.

This is an internal repository disposition against the pinned external
revision. It is not upstream review, acceptance, certification, adoption, or
merge approval.

## 2. Review baseline and independence

The M028 implementation/test head was `a65eecb`:

- `a65eecb` — `fix: isolate I2PControl address book state`
- baseline before M028: `03a384aec495232e64468dcf61d60dd2bab5cfe0`

The current `master` head reviewed by M029 is `c51a8616184aeb43fbacbe3240aeda956d59cf75`.
The heads are not identical. The only post-freeze commit is:

- `c51a861` — `docs: close M028 address book isolation`

That commit was explicitly reviewed and contains documentation/closure records
only; it contains no unreviewed production change. The M028 frozen production
and test implementation remains unchanged.

The reviewer is this distinct Codex review run on 2026-08-04. It is separate
from the prior implementation run that produced `a65eecb`; no organizational
upstream review was requested or used.

## 3. External contract pin

Read-only refetch on 2026-08-04 confirmed:

| Source | Title | Status | Created | Last updated | Source location |
|---|---|---|---|---|---|
| Proposal | `I2PControl Expansion`, Proposal 170 | Open | 2026-05-20 | 2026-05-20 | https://i2p.net/en/proposals/170-i2pcontrol-expansion/ |
| Existing API contract | `I2PControl JSON-RPC` | Documentation page; no proposal status | not stated | 2025-10 | https://i2p.net/en/docs/api/i2pcontrol |

The Proposal 170 revision is unchanged from the pinned 2026-05-20 revision, so
no contract-rebase plan is required. The existing API page still documents the
`API`/`Password` authentication flow, `params.Token`, JSON-RPC behavior, and
the `-32001` through `-32006` I2PControl error inventory.

## 4. Final changed-file classification

The complete M028-to-current inventory was reviewed from baseline
`03a384a`:

Production implementation and permitted composition/dependency seams:

- `emissary-cli/Cargo.toml` — optional `serde_json` feature ownership;
- `emissary-cli/src/address_book.rs` — one legacy/enabled owner boundary,
  durable state, and focused unit evidence;
- `emissary-cli/src/main.rs` — runtime-enabled production composition;
- `emissary-cli/src/i2pcontrol/production.rs` — dedicated administrative
  adapter and existing production state wiring;
- `emissary-cli/src/i2pcontrol/server.rs` — existing method composition.

Focused tests:

- `emissary-cli/tests/adversarial.rs`;
- `emissary-cli/tests/production_adapter.rs`;
- `emissary-cli/tests/production_composition.rs`;
- focused unit tests in `emissary-cli/src/address_book.rs`.

Directly affected documentation and planning authority:

- `docs/i2pcontrol/README.md`;
- `docs/i2pcontrol/address-book.md`;
- `docs/i2pcontrol/proposal-170-support.md`;
- `docs/i2pcontrol/proposal-170-conformance.md`;
- `plans/closure/i2pcontrol-proposal-170/027-closure-invalidation.md`;
- `plans/closure/i2pcontrol-proposal-170/028-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/028-implementation-disposition.md`;
- `plans/implementation/i2pcontrol-proposal-170/028-post-m027-status-and-addressbook-feature-isolation.md`;
- `plans/implementation/i2pcontrol-proposal-170/029-in-scope-conformance-reclosure.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/registry.md`;
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`.

No changed file contains router algorithm, transport, NetDB, peer-selection,
streaming, LeaseSet, cryptographic, frontend, new SAM, new dependency,
data-plane, CI/release, fuzz/soak, or upstream-contribution work. No M029
production implementation pass was needed.

## 5. Requirement-to-evidence matrix

| Dimension | Evidence | Result |
|---|---|---|
| Wire | M020–M027 retained tests; enabled full suite; literal Proposal 170 fixtures; conformance manifest | pass |
| Source | RouterInfo contract and source map; 16 available, 1 neutral, 26 unavailable; ClientServicesInfo and AddressBook composition tests | pass |
| Runtime | Production composition; startup inventory; proxy/SAM/I2CP observation; explicit unsupported registry for all missing tunnel data planes | pass |
| Persistence | AddressBook current/backup recovery and transition tests; TunnelManager atomic publication, rename/failure tests, and full suite | pass |
| Feature isolation | No-feature build; runtime-disabled stale-state test; enabled composition; disable/re-enable restart test; optional dependency guard | pass |
| Security | Authentication order, path confinement, bounds, redaction, secret-safe raw configuration, temporary cleanup, and resource-free unsupported backend review | pass |
| Evidence | Independent literal fixtures, negative tests, production composition, restart/transition tests, source inspection, and exact command outcomes below | pass |
| Governance | M019 superseded; M027 invalidation retained; M028 closed; M029 controlling; registry/roadmap/support docs reconciled; internal-only boundary retained | pass |

Base I2PControl and JSON-RPC behavior remains exact: `Authenticate` accepts
`API` and `Password` without a mandatory username, the response `API` is
numeric, protected requests accept `params.Token`, header conflicts fail
closed, notifications execute without a response, explicit null IDs remain
IDs, invalid IDs are not coerced, and direct base RouterInfo selectors remain
available. The six authentication/version error codes remain distinct.

AddressBook has one owner when enabled and no Proposal 170 control-state owner
when the feature is absent or runtime-disabled. The four execution states are
covered by code inspection and focused tests: no-feature, compiled-disabled,
enabled, and disable/re-enable across restarts. Disabled mode leaves current,
backup, and temporary control-state files untouched and out of lookup;
enabled mode restores the retained state without a second authority.

TunnelManager retains seven lowercase actions, twelve exact types, canonical
result shapes, strict validation, one-publication edit/rename, failure-atomic
prior state, startup collision protection, secret omission, and deterministic
resource-free unsupported lifecycle behavior.

ClientServicesInfo retains six direct presence selectors, startup/control-plane
I2PTunnel provenance, actual HTTP/SOCKS task-exit state, bounded SAM incomplete
and recovery states, actual I2CP listener state, and exactly `BOB: false`.

RouterInfo retains exactly 43 additions and the 16/1/26 disposition. Available
rows have bounded production owners; clock skew `null` is protocol-permitted;
unavailable rows fail sanitized before assembly; mixed, oversized, source-fail,
and response-bound requests do not return partial or fabricated results.

## 6. Verification executed

All commands below were run against current `master` after the post-freeze
documentation-only commit was reviewed.

| Command | Outcome |
|---|---|
| `cargo check -p emissary-cli --no-default-features` | pass |
| `cargo test -p emissary-cli --no-default-features address_book` | pass, 18 |
| `cargo test -p emissary-cli --no-default-features` | pass, 54 |
| `cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings` | pass |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol address_book` | pass, 233 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol tunnel_manager` | pass, 139 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol client_services` | pass, 87 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol router_info` | pass, 104 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol production_composition` | pass, 1 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol conformance_manifest` | pass, 6 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol m027_literal_fixtures` | 0 matched; corrected integration-target invocation used |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition` | pass, 8 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter` | pass, 20 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest` | pass, 58 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m027_literal_fixtures` | pass, 7 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test client_services_integration` | pass, 15 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test golden_fixtures` | pass, 44 |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | pass, 1,219 |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass |
| `cargo check -p emissary-core` | pass |
| `cargo clippy -p emissary-core --all-targets -- -D warnings` | pass |
| `cargo +nightly fmt --all -- --check` | unrelated pre-existing diff in `examples/rust-tutorial/src/main.rs` |
| targeted `rustfmt +nightly --check` on all seven M028 Rust files | pass |
| `git diff --check` | pass |

The required `cargo test -p emissary-core` test target does not currently
compile on this repository baseline. A bounded follow-up exposed unrelated
test-only errors: missing `channel`/`UPDATE_INTERVAL` in `events.rs` and stale
two-argument `EventManager::new` calls in SAM/transport tests. M028 changed no
core file, so this is isolated baseline debt and does not weaken changed-path
evidence; no prohibited core repair was introduced.

## 7. Failure, restart, cancellation, and contention review

- Enabled AddressBook current/backup corruption retains fail-closed recovery;
  disabled mode neither reads nor fails on stale/corrupt control-state files.
- Enable, disable, and re-enable are restart-based transitions. Disable
  preserves state on disk and ignores it; re-enable restores it through the
  single owner without duplicate authority.
- Download failure preserves the existing legacy warning/retry and persistence
  path while enabled downloads merge only through the active control owner.
- TunnelManager publication and rename failure preserve the prior durable
  generation; a lost response after durable commit does not imply rollback.
- Startup-owned tunnel name collisions remain mutation-rejected and read-only.
- Proxy task exit and generation fencing clear inactive state rather than
  leaving a stale running report; SAM incomplete/recovery observations remain
  bounded; I2CP reports actual listener state.
- RouterInfo source failure, mixed unavailable requests, and response bounds
  fail before partial assembly and use sanitized errors.
- Cancellation before publication leaves the previous state; cancellation after
  publication preserves the committed state. Existing serialized mutation
  ownership prevents concurrent readers/mutators from observing an uncommitted
  generation, and code inspection found no new lock-across-await path.

No new long-running harness was added; deterministic retained tests and code
inspection cover these cases.

## 8. Compatibility, migration, and security review

Existing router configuration, legacy `addresses`/`destinations` files, and
enabled M022 control-state snapshots remain compatible. No schema migration or
wire migration was introduced. Configuration-shaped values remain inert and
cannot select arbitrary files. The ordinary AddressBook handle exposes lookup,
not Proposal 170 mutation authority.

Authentication occurs before protected work. AddressBook paths are confined,
collections and responses are bounded, state publication uses temporary-file
cleanup and current/backup atomicity, and errors are sanitized. Passwords,
tokens, proxy/outproxy/IRC credentials, private keys, private destinations,
raw control-state values, and private paths are not emitted through ordinary
logs, responses, or representative fixtures. Sensitive tunnel fields retain
redacted representations. Unsupported backends open no listeners, sessions,
destinations, tasks, or traffic resources and never report running.

## 9. RouterInfo and tunnel-runtime disposition

The exact RouterInfo source matrix is unchanged at 16 available, 1
protocol-permitted neutral, and 26 unavailable. M029 does not reopen M026 or
authorize new telemetry, polling, NetDB inspection, peer classification, or
fabricated values.

All twelve tunnel types remain registered. Missing HTTP, IRC, SOCKS-IRC,
CONNECT, Streamr, bidirectional, and other data planes are explicit
unsupported runtimes: definitions may be inspected and persisted, but start
and restart return deterministic not-implemented outcomes, stop is safe, and
no runtime resource is allocated.

## 10. Documentation, registry, and future-plan disposition

The following now identify M029 as the controlling internal closure:

- `docs/i2pcontrol/README.md`;
- `docs/i2pcontrol/address-book.md`;
- `docs/i2pcontrol/proposal-170-support.md`;
- `docs/i2pcontrol/proposal-170-conformance.md`;
- `plans/registry.md`;
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- this closure record.

M019 and M019A remain superseded/invalidated historical evidence. M020–M027
remain retained corrective evidence. M027's invalidation remains historical;
M028 is closed implementation evidence; M029 is the controlling final review.

M029 was the only registered downstream handoff. It is now closed, and no
future registered plan can be unblocked by this result. Missing RouterInfo
sources and missing tunnel data planes remain explicitly out of scope and
would require separately authorized plans if pursued.

## 11. Unresolved findings

No unresolved M029 high, medium, or low finding remains in the changed scope.
The unrelated core test-compilation debt and untouched tutorial formatting
drift are recorded as verification caveats, not M029 implementation findings.

## 12. Internal-only attestation

The review and all repository writes remained within `eggstack/emissary`.
External specifications were inspected read-only. No upstream issue, pull
request, review request, discussion, submission, adoption request, merge
solicitation, maintainer contact, or contribution artifact was created or
prepared.
