# M104 Closure — Full Proposal 170 Live Interoperability and Reclosure

Status: **closed as blocked**

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/104-full-proposal-170-live-interoperability-and-reclosure.md`

Review date: 2026-08-28

## 1. Disposition

M104 executed its authorized final verification and reconciliation work, then
stopped at the plan's mandatory residual-blocker condition. The repository does
not claim full Proposal 170 support. The final disposition is **blocked**: the
M095 matrix still contains 164 applicable `blocked_primitive` cells, including
shared-session, SAM-wire, destination/key lifecycle, private-key import, proxy
management, server presentation/routing, and LeaseSet security primitives.

This is an internal closure against Proposal 170 revision `2026-05-20`, whose
upstream status remains `Open`. It is not upstream interoperability certification.

## 2. Exact heads and dependency gate

Production implementation head reviewed: `cf5f5192c97d9e9963d9128f772436625a38a6c6`
(`docs(i2pcontrol): close M099 implementation`). M104 introduced no production
code, dependency, schema, or data-plane changes; this closure and the associated
planning-state corrections are the only changes in this pass.

| Dependency | Closure/evidence | Result |
|---|---|---|
| M095 | `plans/closure/i2pcontrol-proposal-170/095-closure.md`; authoritative matrix | closed; exact inventory retained |
| M096 | `plans/closure/i2pcontrol-proposal-170/096-closure.md` | closed; 13 SetConfig keys operational |
| M097 | `plans/closure/i2pcontrol-proposal-170/097-closure.md` | closed as blocked; common session/key residuals remain |
| M098 | `plans/closure/i2pcontrol-proposal-170/098-closure.md` | closed; independent client/proxy/HTTP slice applied |
| M099 | `plans/closure/i2pcontrol-proposal-170/099-closure.md` | closed internally, partial; independent server slice applied |
| M100 | `plans/closure/i2pcontrol-proposal-170/100-closure.md` | closed |
| M101 | `plans/closure/i2pcontrol-proposal-170/101-closure.md` | closed |
| M102 | `plans/closure/i2pcontrol-proposal-170/102-closure.md` | closed |
| M103 | `plans/closure/i2pcontrol-proposal-170/103-closure.md` | closed |
| revised M098/M099 residual transfer | M098 and M099 closure records plus matrix | explicit residual ownership; no cell silently accepted |
| zero residual option gate | M095 matrix and static guard | **failed: 164 blocked cells** |

The residual line has no dependency-ready successor. The missing owners are not
available within the accepted containment boundary, and the plans prohibit
vendoring/forking Yosemite, adding Proposal-170-shaped core APIs, or adding a
dependency merely to make the matrix green.

## 3. Final machine-readable matrix

The authoritative file is
`plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`.

- SHA-256: `fcc7d21dd886cd96ac614507abba5e3cfc806cee942ebbb09eb387e1a60078ac`
- RouterInfo: 43 rows — 42 available, 1 protocol-permitted neutral, 0 unavailable
- AddressBook SetConfig: 13 rows — all M096-owned and operational/metadata-valid
- TunnelManager: 70 canonical option rows × 12 canonical types = 840 cells
- Cell totals: 218 `apply`, 164 `blocked_primitive`, 458 `not_applicable`
- `planned_apply`: 0; unknown/unsupported/accept-inert: 0
- Canonical actions: 7
- ClientServicesInfo selectors: 6

The blocked cells are distributed as follows:

| Residual family | Applicable blocked cells | Owner/blocker |
|---|---:|---|
| `Shared` | 7 | M097; bounded shared-session ownership and handoff |
| `UseSSL` | 4 | M097; Yosemite SAM session wire |
| `TunnelVariance`, `TunnelBackupQuantity`, `SigType`, `CustomOptions` | 40 | M097; Yosemite SAM session wire |
| `NewDest`, `PersistentClientKey` | 14 | M097; destination/key lifecycle |
| `PrivKeyFile` | 10 | M097; confined validated private-key import/store/handoff |
| `UseOutproxyPlugin`, `SSLProxies`, `JumpList` | 12 | residual line; no bounded accepted owner |
| `ConnectDelay`, `Profile`, `DelayOpen`, `Reduce*`, `Close*` | 56 | residual line; no exact client session lifecycle owner |
| `AllowInternalSSL`, `UniqueLocalAddressPerClient`, `MultiHoming` | 6 | residual line; TLS/address-routing owners absent |
| `EncryptLeaseSet`, `OptionalLookup`, `LeaseSetClientAuths` | 15 | residual line; LeaseSet serializer/key handoff absent |

## 4. RouterInfo evidence

The 43-row inventory, source owners, neutral semantics, bounds, and fixtures are
retained by M095 and M100-M103. The M104 live test exercised the production
RouterInfo composition for the available source groups and deliberately observed
the no-source network-error/transit behavior in the local no-reseed fixture. It
did not claim a full dynamic 43-row public-network pass because the M104 gate
failed before such certification could be truthful. The M103 by-design-empty
banned-peer proof remains accepted and no router-wide ban behavior was added.

## 5. AddressBook and ClientServicesInfo evidence

M096 remains the authority for authenticated CRUD, subscriptions, all 13
SetConfig keys, persistence, publication, path confinement, and transactional
failure behavior. The live production-composition test re-exercised authenticated
add/lookup/delete, subscription handling, invalid path rejection, persistence,
and RouterInfo AddressBook projection.

M027/M038 and the current feature suite remain the authority for the exact six
ClientServicesInfo selectors: `I2PTunnel`, `HTTPProxy`, `SOCKS`, `SAM`, `BOB`,
and `I2CP`. The live fixture re-exercised the production method and its direct
presence behavior; no selector or response-shape change occurred in M104.

## 6. TunnelManager and data-plane evidence

The registry and static guards retain exactly 12 tunnel types and 7 canonical
actions. M066-M071/M093 remain the accepted implementation and security evidence
for the twelve real runtime families:

`client`, `httpclient`, `ircclient`, `socks`, `socksirc`, `connectclient`,
`streamrclient`, `server`, `httpserver`, `httpbidirserver`, `ircserver`, and
`streamrserver`.

The M104 live fixture re-exercised production TunnelManager creation, canonical
configuration access, occupied-listener start failure, edit/restart recovery,
and bounded shutdown. It did not claim all-option or all-family traffic evidence:
the 164 blocked cells make that acceptance requirement unsatisfiable at this
head. No blocked option is silently allocated or reported as applied.

## 7. Live environment

Topology: one feature-enabled Emissary child process, real HTTPS/TLS
I2PControl listener on loopback, real production AddressBook and TunnelManager
composition, loopback HTTP client, and the process's real SAM listener. The
fixture intentionally uses `reseed_threshold = 0` and no external Java I2P/i2pd
peer, so it is bounded local production evidence rather than public-network
certification.

Command:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
```

Result: pass, 1 test, 20.82s. The test covered real TLS/authentication, request
IDs and notifications, AddressBook mutation/path guards, RouterInfo and service
queries, and TunnelManager bind-failure/restart/shutdown recovery.

## 8. Security, failure, persistence, and containment reclosure

No M104 production path changed. M093 remains the security authority for trusted
peer identity, admission transactionality/cardinality, literal-loopback target
confinement, HTTP/IRC/Streamr bounds, secret/path redaction, and cancellation.
M097-M099 closure evidence remains authoritative for fail-before-allocation and
no-downgrade behavior. The live fixture rechecked bind/start recovery, durable
AddressBook state, and deterministic child shutdown. The full M104 mixed-inventory,
all-family traffic, and reference-router evidence is explicitly not claimed.

Containment is unchanged: Proposal 170 policy remains in
`emissary-cli/src/i2pcontrol/**`; no core/util/dependency/frontend/workflow path
was modified. The M062 containment suite passed, and the feature-disabled/default
build received no I2PControl-only production change. Base methods outside
Proposal 170 remain out of scope.

## 9. Verification outcomes

| Command | Outcome |
|---|---|
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass |
| `cargo check` | pass; default build remains feature-isolated |
| `cargo test -p emissary-core` | one full-run ML-KEM failure; exact failing test passed on isolated rerun |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | pass; 1,756 tests across 25 suites |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment` | pass; 26 tests across 2 suites |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture` | pass; 1 test |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m062_dependency_containment` | pass; 20 tests across 2 suites |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | fails on pre-existing stable/nightly rustfmt option mismatch; no formatter-only churn retained |
| `git diff --check` | pass |

The isolated lower-layer rerun was:

```text
cargo test -p emissary-core --test ml_kem client_ml_kem_1024_server_ml_kem_768 -- --exact --nocapture
```

Result: pass, 1 test, 15 filtered out. This does not convert the first full-suite
failure into an M104 pass and does not authorize changes outside the plan.

## 10. Changed-path review

M104 changed only planning/documentation records:

- this closure record;
- M104 status line;
- registry M099/M104 handoff reconciliation;
- full-support implementation README;
- full-support roadmap status/closure text;
- `docs/i2pcontrol/README.md` status text;
- `docs/i2pcontrol/proposal-170-conformance.md` status text;
- `docs/i2pcontrol/proposal-170-support.md` status and residual-reclosure text.

No production source, Cargo manifest/lockfile, dependency, schema, frontend, or
external source was changed. The M095 matrix hash above is the final reviewed
machine-readable head.

## 11. Future-plan disposition

M099 was already closed internally and its stale `ready` registry entry was
corrected. M104's closure does not newly unblock any future implementation plan.
There is no registered dependency-ready residual plan: the residual cells still
require a bounded shared-session/SAM-wire/destination-key/private-key/LeaseSet or
safe presentation owner that is absent under the accepted containment rules.

The full-support roadmap remains active with the residual option blocker as its
current handoff. A future numbered corrective plan may be registered only after
repository/dependency evidence demonstrates such a bounded path. A new M104
attempt may proceed only after that plan closes and the matrix contains no
applicable `blocked_primitive` or `planned_apply` cell.

## 12. Unresolved findings and final attestation

- High/medium M104 findings introduced by this pass: none.
- Low: one non-deterministic lower-layer ML-KEM failure in the broad core test;
  isolated rerun passed and no M104 code changed.
- Blocking correctness finding: 164 applicable residual TunnelManager cells.

Base I2PControl methods outside Proposal 170 are explicitly out of scope. All
external specifications, dependency sources, and reference-router material were
read-only evidence. No upstream branch, issue, pull request, review request,
submission, merge/adoption request, release, maintainer contact, or external
service was modified.

Final disposition: **blocked** (M104 closed as blocked; Proposal 170 support
remains partial against the pinned revision).
