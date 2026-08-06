# I2PControl Proposal 170 Milestone M044 — Corrective Final-Head Reclosure

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/044-corrective-final-head-reclosure.md`

Corrective authority:

- `plans/closure/i2pcontrol-proposal-170/039-closure-invalidation.md`

Reviewed final implementation/evidence head: `342420e`

Dependency closure heads:

- M040: `c316487`
- M041: `f7a9b37`
- M042: `ef30155`
- M043: `342420e`

Pinned Proposal 170 revision: `2026-05-20`, unchanged and still Open.

Review date: 2026-08-06

## 1. Final disposition

M044 independently reviewed the corrected final head and restores the truthful
subsystem status to `partial Proposal 170 support`.

The three invalidated defects are corrected and no high- or medium-severity
correctness, security, ownership, compatibility, containment, persistence, or
evidence defect remains in an implemented dimension. This is not full Proposal
170 completion: ten tunnel families remain explicit unsupported runtimes and 26
RouterInfo additions remain unavailable because no bounded canonical Emissary
owner exists.

## 2. Requirement-to-evidence matrix

| Requirement | Exact evidence | Result |
|---|---|---|
| Authentication and token behavior | `auth.rs`, `server.rs`, golden/adversarial/package tests | pass |
| Normalized/atomic failed-auth throttle | `reserve_failure`; IP-port, barrier, reset, capacity tests | pass |
| Direct/base compatibility | RPC inventories, conformance, golden, M027 literal fixtures | pass |
| RouterInfo contract/source matrix | `router_info.rs`, source map, truthfulness tests | pass: 16 available, 1 neutral, 26 unavailable |
| AddressBook entries/subscriptions/configuration | owner/runtime modules, AddressBook tests, adapter, live runtime | pass/qualified |
| TunnelManager wire and lifecycle | tunnel manager tests, M033, production adapter/composition | pass |
| Startup server preservation | actual `ServerTunnelManager` fake-SAM regression | pass: session, destination, forward, liveness |
| Generic client/server backends | production composition, backend lifecycle, M033, live runtime | pass/formation qualified |
| Unsupported tunnel families | backend registry, resource-free guards, fixtures | pass: ten explicit unsupported |
| Startup ownership | production inventory and M033 ownership tests | pass; no adoption/control handle |
| StartOnLoad and failure recovery | M033, production composition, live restart evidence | pass |
| ClientServicesInfo and SAM observation | service registry tests, production adapter, live runtime | pass/qualified |
| Persistence/recovery/durability | generation/server/address-book stores and persistence suite | pass with documented directory-sync qualification |
| Feature isolation | no-feature package suite and static guards | pass |
| Secret handling and resource bounds | security/adversarial/static guard suites | pass |
| Containment | M037 containment test and final changed-path review | pass |
| Focused/live runtime validation | M040–M043 regressions and M038 live child process | pass/qualified for no peers/downloader |
| Internal-only/no-upstream compliance | repository log and planning attestation | pass |

## 3. Independent verification

The following representative commands were rerun against `342420e`:

- `cargo check -p emissary-cli --no-default-features` — pass;
- `cargo test -p emissary-cli --no-default-features` — pass, 56 tests;
- `cargo check -p emissary-cli --no-default-features --features i2pcontrol` — pass;
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition` — pass, 8 tests;
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m033_tunnel_lifecycle` — pass, 3 tests;
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture` — pass, 1 test;
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol` — pass;
- `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` — pass;
- `cargo test -p emissary-core sam` — pass, 145 unit-filtered and 4 integration tests;
- `git diff --check` — pass.

The live test records no reseeded peer set for client/server traffic formation
and no composed positive HTTP downloader. Those are environmental
qualifications, not fabricated success or corrective defects. Stable rustfmt
continues to report the repository's known nightly-only option mismatch; no
unrelated formatter changes are retained.

## 4. Changed-path and scope review

From the invalidated baseline, corrective production behavior is confined to:

- `emissary-cli/src/tunnel/server.rs` — startup cancellation-owner lifetime;
- `emissary-cli/src/i2pcontrol/auth.rs` and `server.rs` — source-IP atomic throttle;
- `emissary-cli/src/address_book.rs` — post-commit refresh result boundary.

Documentation and test-only evidence changes are confined to the affected
I2PControl/tunnel/AddressBook paths and planning records. No new core behavior,
tunnel family, RouterInfo source, frontend, workflow, release automation,
scheduler, second owner, arbitrary path, or upstream artifact was introduced.

## 5. Future-plan disposition

M044 is the final registered corrective handoff and is now closed. No future
implementation plan can be unblocked: deferred RouterInfo sources and the ten
unsupported tunnel families remain outside this roadmap and have no accepted
dependency-ready owner. The invalidated M039 record remains historical and is
not rewritten.

## 6. Internal-only attestation

External Proposal 170 material was accessed read-only. No upstream repository,
issue, pull request, review, merge/adoption/submission channel, or maintainer
channel was mutated. No upstream review, merge, adoption, submission, or
contribution artifact was requested or prepared.

**Disposition: `partial Proposal 170 support`; M044 closed.**
