# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 production support; M109, M115, M110, M116, M117, M118, M111, and M112 are closed; M112 closed as blocked with 45 named residual cells; M113-M114 remain roadmap-defined and blocked; M095 currently records `312 apply / 70 blocked_primitive / 458 not_applicable`

This directory contains bounded internal implementation, audit, corrective, and closure handoffs for the I2PControl Proposal 170 subsystem.

Authoritative references:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`
- ADR-0001/0002/0003/0004
- ADR-0005 — internal Yosemite fork dependency boundary
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`
- `061-containment-boundary.toml`
- `062-dependency-containment.toml`
- `095-full-support-matrix.toml`
- `105-residual-option-audit.toml`
- `110-completion-ledger.toml`
- `plans/closure/i2pcontrol-proposal-170/116-closure.md`
- `plans/closure/i2pcontrol-proposal-170/117-closure.md`
- `plans/closure/i2pcontrol-proposal-170/118-closure.md`
- `plans/closure/i2pcontrol-proposal-170/111-closure.md`
- `plans/registry.md`

Pinned Proposal 170 revision: `2026-05-20` (proposal remains Open).

## Internal-only rule

All work is internal to `eggstack/emissary`. External specifications, I2P/Java I2P/i2pd/I2P+/Yosemite/Tokio source or documentation, issues, commits, pull requests and reference routers are read-only evidence.

No plan authorizes upstream submission, review request, maintainer contact, contribution preparation, merge/adoption request, issue/PR mutation, branch/tag push, release, or repository write outside this fork.

## Scope and containment

Preferred production ownership remains `emissary-cli/src/i2pcontrol/**`.

M061/M062/M063 remain containment authority. M109/M115 are the bounded historical exception for existing CLI startup tunnel ownership. M116 adds **no** non-I2PControl production authority.

No standalone crate split, router-core API, Yosemite patch/vendor, parallel SAM stack, frontend coupling, or hosted CI expansion is authorized by M116. M117 separately closes the ADR-0005-authorized optional, exact-revision I2PControl-only Yosemite alias and adapter integration. M118 is the separately registered neutral SAM/tunnel-pool exception for generic variance and standby semantics; it does not change I2PControl production ownership or matrix counts.

## Current production state

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable.
- AddressBook CRUD, subscriptions, all 13 SetConfig keys, cross-book shadowing and lookup coherence are operational.
- Exactly 12 Proposal 170 tunnel types have real backends.
- Exactly seven canonical TunnelManager action handlers exist.
- All six ClientServicesInfo selectors are operational.
- API 1-only authentication and M107/M108 managed TLS hardening are operational.
- M109/M115 startup lifecycle/runtime-disable work is closed.
- M110 added real I2PControl-local shared sessions, destination identity ownership and confined key import.
- Post-M110 review found and M116 corrected: shared-session lost wakeup, creator-cancellation poisoning, collision-unsafe identity fingerprinting, Streamr cross-producer delivery, unproven `NewDest` lifecycle semantics, and raw internal secret `Debug` derivation.
- Full public/reseeded/reference-router certification remains open.

Current M095 matrix is:

- 312 `apply`;
- 70 `blocked_primitive`;
- 458 `not_applicable`;
- 0 planned/unknown/unsupported/accept-inert cells.

M116 closure records the exact reclassification: seven client `NewDest` cells moved from `apply` to `blocked_primitive` under M112.

Official status remains **partial Proposal 170 support**.

## Current/future handoff sequence

| Handoff | Status | Scope |
|---|---|---|
| M095 | closed | exact full-support matrix/containment budget |
| M096 | closed | all 13 AddressBook SetConfig keys |
| M097 | closed as blocked | common session/key safe subset; residuals retained |
| M098 | closed | client proxy/outproxy/auth/privacy subset |
| M099 | closed internally — partial | server access/filter/admission/rate subset |
| M100 | closed | transit source |
| M101 | closed | signed router-news source |
| M102 | closed | network-error owner |
| M103 | closed | banned-peer source disposition |
| M104 | closed as blocked | prior final reclosure stopped on residual option cells |
| M105 | closed | residual primitive/applicability audit |
| M106 | closed | six TCP-client DelayOpen cells |
| M107 | closed | API1/AddressBook/fresh managed-TLS corrective |
| M108 | closed | managed TLS upgrade-permission corrective |
| M109 | closed | startup-managed named lifecycle + `All=true` semantics |
| M115 | closed | M109 runtime-disable/lifecycle corrective |
| M110 | closed historically; M116 corrective required | shared client session + destination/key/PrivKeyFile ownership |
| **M116** | **closed** | M110 concurrency/cancellation/identity/Streamr/NewDest/secret corrective; closure recorded |
| **M117** | **closed** | ADR-0005-authorized exact Yosemite fork pin and I2PControl adapter integration; no matrix promotion |
| **M118** | **closed** | neutral SAM tunnel variance and standby/failover capability; no matrix promotion |
| **M111** | **closed** | Yosemite SAM session-wire completion; 40 SessionWire cells applied, four UseSSL cells remain explicitly blocked |
| M112 | closed as blocked | six TCP client families apply `ConnectDelay`, `Close`, `CloseTime`, and `NewDest`; 45 M112 residual cells remain blocked |
| M113 | proposed / blocked | server presentation/routing/LeaseSet; up to 21 cells |
| M114 | proposed / blocked | zero-residual live/reference final reclosure |

M110-M114 were reserved before the later M115/M116 correctives. Execution order is M109 → M115 → M110 → M116 → M111 → M112 → M113 → M114.

Plans in this completion line:

- `109-startup-managed-tunnel-action-semantics-corrective.md`
- `110-shared-client-session-and-destination-key-ownership-completion.md`
- `111-sam-session-wire-option-completion.md`
- `112-client-proxy-and-session-lifecycle-residual-completion.md`
- `113-server-presentation-address-routing-and-leaseset-residual-completion.md`
- `114-full-proposal-170-live-interoperability-and-final-reclosure.md`
- `115-m109-runtime-disable-and-lifecycle-truthfulness-corrective-pass.md`
- `116-m110-shared-session-and-newdest-corrective-pass.md`
- `118-neutral-sam-tunnel-pool-variance-backup-capability.md`

Per `plans/003-planning-process.md`, M112 is closed as blocked. M113-M114 retain
their independent blockers; no future Emissary handoff is dependency-ready from
this closure.

M118 closure: `plans/closure/i2pcontrol-proposal-170/118-closure.md`. It records the
read-only reference freeze, exact neutral owner paths, standby promotion/replenishment
semantics, verification outcomes, and the decision that M111 could execute.

## M116 — closed corrective

M116 does not reopen general Proposal 170 scope. It corrects M110's exact owner boundaries:

- shared stream/datagram acquisition must be linearizable and lost-wakeup-free;
- creator cancellation/drop/failure must not strand a compatibility key;
- persistent identity compatibility must use collision-safe secret-redacted equality rather than a 64-bit `DefaultHasher` fingerprint;
- shared Streamr delivery must route only authenticated traffic from each member's configured producer;
- unrelated Streamr peers must not be forwarded;
- `NewDest` must be re-derived from pinned/reference semantics;
- if correct `NewDest` requires M112's `Close*` lifecycle primitive, those cells return to `blocked_primitive` and transfer to M112 without implementing M112 here;
- secret-bearing internal store/session types must not expose raw private material through ordinary `Debug`/`Display`;
- matrix/ledgers are reconciled to exact post-corrective evidence in `plans/closure/i2pcontrol-proposal-170/116-closure.md`.

M116 production authority is I2PControl-only and explicitly enumerated in the plan.

## Residual option ownership during M116

The pre-M116 matrix has 127 blocked cells owned nominally by:

- M111: 4 — `UseSSL` remains blocked;
- M112: 69 — proxy/plugin/jump, client lifecycle `Reduce*`/`Close*`, and seven `NewDest` cells;
- M113: 21 — server presentation/address-routing/LeaseSet.

M116 returned all seven client `NewDest` cells to `blocked_primitive` under M112. `Shared × streamrclient` remains `apply` because canonical authenticated producer matching is implemented within the existing owner.

A cell becomes/remains `apply` only with request→real-runtime evidence. Difficulty is not evidence of `not_applicable`, and no accept-inert state is permitted.

## M111 — SAM session-wire options

M111 is closed. M117 supplies the accepted internal generic Yosemite API through ADR-0005 and M118 supplies the neutral variance/backup runtime effect. `TunnelVariance`, `TunnelBackupQuantity`, `SigType` (router-supported type 7), and bounded `CustomOptions` reach the real Yosemite `SESSION CREATE` path for all 40 applicable cells. `UseSSL` remains blocked because Proposal local application/session TLS is not Yosemite SAM-control TLS; no accepted Emissary owner exists. No raw/parallel SAM stack or Proposal-shaped core seam was added. Closure: `plans/closure/i2pcontrol-proposal-170/111-closure.md`.

## M112 — client proxy/lifecycle residuals

M112 is closed as blocked. It applied portable `ConnectDelay`, `Close`, `CloseTime`,
and `NewDest` behavior for six TCP client families. Proxy/plugin/TLS-jump,
`Profile`, `Reduce*`, and Streamr lifecycle cells remain explicitly blocked; see
`plans/closure/i2pcontrol-proposal-170/112-closure.md`.

## M113 — server presentation/LeaseSet residuals

M113 remains blocked and security-sensitive. LeaseSet encryption/client authorization requires real accepted primitives with no downgrade. Presentation/address-routing may not relax literal-loopback/no-SSRF boundaries.

## M114 — final reclosure

M114 remains blocked until M116 and M111-M113 are closed as applicable, the authoritative matrix has zero unresolved applicable cells, and no high/medium Proposal-scoped corrective remains.

M114 implements no missing feature. Only successful M114 closure may state `full Proposal 170 support against pinned revision 2026-05-20`.

## Security invariants retained

- trusted remote identity is Yosemite-derived;
- Streamr application delivery must be producer-isolated;
- shared-session compatibility must never cross incompatible security identities;
- server admission remains bounded/transactional;
- server local targets remain confined/literal-loopback under current authority;
- HTTP/IRC filters remain non-bypassable;
- direct I2P proxy traffic never falls through to clearnet DNS;
- clearnet proxy traffic requires explicit I2P outproxy;
- secret/key/path values remain redacted/confined;
- managed I2PControl key material retains M107/M108 protections;
- LeaseSet security never silently downgrades;
- lifecycle workers/tasks remain generation-local and cancellable;
- feature-compiled but runtime-disabled startup remains historical after M115.

## Verification policy

Use focused deterministic M116 regressions plus existing feature-gated containment/matrix/live-runtime suites. Do not add a CI farm.

Baseline:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

The known stable/nightly rustfmt mismatch remains a tooling issue; record it rather than retaining unrelated formatter churn.

## Closure discipline

M116 closes only through a new closure record with exact changed paths, F1-F6 requirement/evidence mapping, deterministic concurrency/cancellation tests, Streamr producer-isolation evidence, `NewDest` authority/disposition, secret-redaction evidence, exact matrix/ledger reconciliation, broad verification, containment/security review, unresolved findings, next-handoff decision, and internal-only attestation.

Historical M110 closure remains historical evidence and is not rewritten to hide the corrective.
