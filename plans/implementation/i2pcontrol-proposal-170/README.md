# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 production support; all twelve tunnel runtimes real; M095-M096 and M100-M103 closed; M097 closed as blocked; revised M098 is current handoff; M099 is queued behind M098; M104 remains blocked

This directory contains bounded internal implementation and closure handoffs for the I2PControl Proposal 170 subsystem.

Authoritative references:

- `plans/000-long-term-specification.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`
- ADR-0001/0002/0003/0004
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`
- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml`
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`
- `plans/registry.md`

Pinned Proposal 170 revision: `2026-05-20` (proposal remains Open).

## Internal-only rule

All work is internal to `eggstack/emissary`. External specifications, I2P/i2pd/Java I2P/Yosemite source, issues, commits, and pull requests are read-only evidence.

No plan authorizes upstream submission, review request, maintainer contact, contribution preparation, merge/adoption request, issue/PR mutation, branch/tag push, release, or repository write outside this fork.

## Scope and containment

Preferred production ownership remains `emissary-cli/src/i2pcontrol/**`.

- M061 owns exact source containment.
- M062/M063 own dependency/feature reachability.
- Proposal 170 policy remains in I2PControl wherever technically possible.
- Lower-layer changes are exceptional and require exact pre-authorized owner/path evidence.
- No standalone crate split is required for aesthetics.
- No broad core/router refactor is authorized for API convenience.
- No hosted CI/fuzz/coverage/release expansion is required.

M091 remains the cautionary boundary case: unauthorized vendored Yosemite/core/dependency work was removed by M092. M093 independently reclosed the corrected tunnel production/security state. Corrective option work must not repeat that pattern.

## Current production state

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable.
- AddressBook CRUD, subscriptions, and all 13 SetConfig keys are operational under the confined owner.
- Exactly 12 Proposal 170 tunnel types have real backends.
- Exactly 7 canonical TunnelManager actions are implemented.
- M097 applied the supported `TunnelLength`, `TunnelQuantity`, and typed `EncType` semantics.
- Remaining primitive-dependent option cells fail before allocation rather than being ignored.
- All 6 ClientServicesInfo selectors are implemented.
- Full live/reseeded/reference-router certification remains open.

Overall status remains partial until TunnelManager option parity and M104 close.

## Current corrective handoff sequence

The original M098/M099 plans were prewritten before M097 executed. M097 closure demonstrated that a milestone-wide hard dependency was too coarse: many client/server option cells are owned by existing I2PControl runtimes even though common session/key cells remain blocked.

The plans have therefore been corrected in place because neither M098 nor M099 had executed or closed. No historical closure record was rewritten.

| Handoff | Status | Scope |
|---|---|---|
| M095 | closed | exact full-support matrix and containment budget |
| M096 | closed | all 13 AddressBook SetConfig keys operational |
| M097 | closed as blocked | supported common session options landed; residual shared-session/SAM-wire/client-key/PrivKeyFile primitives remain blocked |
| M098 | **ready — current handoff** | reconcile M098 cells, implement client proxy/outproxy/auth/HTTP and exact generation-local management subset independent of M097 |
| M099 | **blocked on M098 integration order only** | reconcile server cells, implement HTTP presentation/access/filter/admission/rate subset independent of M097 |
| M100 | closed | request-independent transit 15-second source |
| M101 | closed | bounded signed router-news source |
| M102 | closed | neutral v4/v6 network-error owner observation |
| M103 | closed | authoritative by-design-empty banned-peer source |
| M104 | blocked | final live interoperability/security/containment/full-support reclosure after zero residual option blockers |

Plans:

- `095-full-support-contract-matrix-and-containment-budget.md`
- `096-addressbook-setconfig-operational-completion.md`
- `097-tunnel-common-session-and-key-option-completion.md`
- `098-client-proxy-management-and-http-option-completion.md`
- `099-server-access-throttle-and-leaseset-option-completion.md`
- `100-routerinfo-transit-15s-source-completion.md`
- `101-routerinfo-news-source-completion.md`
- `102-routerinfo-network-error-owner-completion.md`
- `103-routerinfo-banned-peer-semantic-completion.md`
- `104-full-proposal-170-live-interoperability-and-reclosure.md`

## M098 corrective revision

M098 is now dependency-ready.

Its first mandatory work package is a cell-level audit of every M098-owned `planned_apply` entry in `095-full-support-matrix.toml` using M097 closure evidence.

A cell remains M098-owned only if exact semantics can be implemented through existing accepted I2PControl client/proxy/filter/runtime ownership. Genuine primitive-dependent cells must be transferred to explicit `blocked_primitive` residual ownership before production code begins.

Expected independent candidates:

- proxy/outproxy routing and policy;
- proxy/outproxy authentication and secret handling;
- direct-I2P/no-clearnet-DNS behavior;
- HTTP User-Agent/Referer/Accept/InternalSSL privacy/filter behavior;
- generation-local client-management semantics only where current runtime ownership is exact.

M098 does not authorize Yosemite changes, new core APIs, dependency/lockfile changes, SOCKS protocol expansion, outproxy plugin architecture, or weakened LAN/anonymity boundaries.

M098 closure must make every cell it still owns `apply`, leave residual blockers explicit, and advance M099.

## M099 corrective revision

M099 is not semantically blocked on M097. It is queued behind M098 only to avoid conflicting edits to the shared matrix, option validator, HTTP/filter surfaces, and registry.

Before production code, it must reconcile every M099-owned server cell plus any server-role cells transferred from M098.

Expected independent candidates:

- HTTP server presentation/filter policy;
- trusted-peer access lists;
- confined filter/access file loading;
- accepted-connection ceilings;
- peer and aggregate rate limits;
- POST limits and timing periods;
- tunnel-local temporary denial.

LeaseSet/session-security options remain blocked when the current supported Yosemite/SAM path cannot implement them without downgrade. `UniqueLocalAddressPerClient` and similar presentation semantics must remain blocked rather than approximated if no safe exact equivalent exists in the accepted literal-loopback server model.

M099 does not authorize new core LeaseSet APIs, router-wide banning, arbitrary filesystem access, request-selected LAN targets, or Yosemite dependency changes.

## Residual option blocker line

M097 closure is the current residual primitive authority.

Named blockers include:

- shared-session ownership/handoff;
- SAM `SESSION CREATE` serialization for several common options;
- client destination/key lifecycle;
- confined `PrivKeyFile` import/store/handoff;
- any M098/M099 cell proven to require the same unavailable authorities.

There is intentionally no speculative successor implementation plan registered today. After M099, inspect the residual matrix and register a new bounded plan only when current repository/dependency evidence supports a solution inside accepted containment.

A missing dependency primitive is not authority to vendor/fork Yosemite or move Proposal 170 policy into core.

## M104 readiness

M104 remains blocked until:

- revised M098 closes;
- revised M099 closes;
- every residual applicable TunnelManager cell is subsequently resolved;
- the matrix contains no applicable `planned_apply`, `blocked_primitive`, unsupported, or unknown cell.

Only then does M104 perform live/reseeded/reference-router interoperability and integrated security/containment reclosure.

## Full-support design rules

### AddressBook

M096 is current authority. All 13 keys are operational with typed persistence, path confinement, bounded downloader integration, and metadata-only theme behavior. Do not reopen it for tunnel option work.

### Tunnel options

Every accepted applicable option must change real runtime behavior. Parser acceptance, rawConfig persistence, or fail-before-allocation rejection is not final support.

Corrective M098/M099 work must transfer genuine blockers explicitly rather than silently retaining milestone-wide dependency edges or approximating unavailable semantics.

### RouterInfo

M100-M103 close the former five source gaps. M104 still independently verifies live/integrated behavior, including real news fetch, transit sampling, network-error observation, and banned-peer semantics.

### ClientServicesInfo

The six-selector implementation remains closed and is regression scope only unless M104 finds a defect.

## Security invariants retained

- trusted peer identity is Yosemite-derived;
- server admission remains bounded/transactional;
- HTTP/IRC filters are non-bypassable;
- server local targets remain confined/literal-loopback as accepted;
- direct I2P proxy traffic never falls through to clearnet DNS;
- clearnet proxy traffic requires an explicit I2P outproxy;
- secrets and path-bearing values remain redacted/confined;
- Streamr state remains bounded;
- tunnel-local temporary denial never becomes router-wide banned-peer state;
- LeaseSet security never silently downgrades;
- M088 lower-layer residual is not reopened absent new evidence.

## Verification policy

Use focused tests plus the existing feature-gated suite and M061/M062 containment guards. Do not add a CI farm.

The historical `m063_feature_reachability` test target is absent in the current checkout; preserve that as a tooling/inventory limitation rather than inventing unrelated scope.

The repository also has a known nightly/stable rustfmt mismatch. Do not create formatter-only churn across audited core files solely to make that gate green.

## Closure discipline

A plan closes only with a closure record containing exact implementation commits, requirement-to-evidence mapping, verification outcomes, security/compatibility/containment review, unresolved findings, and internal-only attestation.

M098/M099 closure does not imply full Proposal 170 support while any residual option cell remains blocked. M104 is the only authority that may finally state full support against the pinned revision.
