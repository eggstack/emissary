# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 production support; all twelve tunnel runtimes real; M095-M096 and M098-M103 closed, M099 closed internally/partial, M097 and M104 closed as blocked, M105 closed; **M106 DelayOpen client-listener handoff is ready**

This directory contains bounded internal implementation, audit, and closure handoffs for the I2PControl Proposal 170 subsystem.

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

All work is internal to `eggstack/emissary`. External specifications, I2P/i2pd/Java I2P/I2P+/Yosemite source, issues, commits, and pull requests are read-only evidence.

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

M091 remains the cautionary boundary case: unauthorized vendored Yosemite/core/dependency work was removed by M092. M093 independently reclosed the corrected tunnel production/security state. Residual option work must not repeat that pattern.

## Current production state

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable.
- AddressBook CRUD, subscriptions, and all 13 SetConfig keys are operational under the confined owner.
- Exactly 12 Proposal 170 tunnel types have real backends.
- Exactly 7 canonical TunnelManager actions are implemented.
- M097 applied the supported `TunnelLength`, `TunnelQuantity`, and typed `EncType` semantics.
- M098 applied the bounded client proxy/outproxy/auth/privacy subset.
- M099 applied the bounded server presentation/access/filter/admission/rate subset.
- Remaining primitive-dependent option cells fail before allocation rather than being ignored or reported as applied.
- All 6 ClientServicesInfo selectors are implemented.
- Full live/reseeded/reference-router certification remains open.

M104's final reviewed TunnelManager matrix contains:

- 218 `apply`;
- 164 applicable `blocked_primitive`;
- 458 `not_applicable`;
- 0 `planned_apply`, unknown, unsupported, or accept-inert cells.

Overall status remains partial until every applicable blocked cell is resolved and a future M104 reattempt closes successfully.

## Current handoff sequence

| Handoff | Status | Scope |
|---|---|---|
| M095 | closed | exact full-support matrix and containment budget |
| M096 | closed | all 13 AddressBook SetConfig keys operational |
| M097 | closed as blocked | supported common session options landed; residual shared-session/SAM-wire/client-key/PrivKeyFile primitives remain blocked |
| M098 | closed | client proxy/outproxy/auth/HTTP slice applied; residual management and unsupported proxy primitives explicitly blocked |
| M099 | closed internally — partial | server presentation/access/filter/admission/rate subset applied; LeaseSet/TLS/address-owner residuals remain blocked |
| M100 | closed | request-independent transit 15-second source |
| M101 | closed | bounded signed router-news source |
| M102 | closed | neutral v4/v6 network-error owner observation |
| M103 | closed | authoritative by-design-empty banned-peer source |
| M104 | closed as blocked | final live interoperability/security/containment/full-support reclosure stopped by 164 residual option blockers |
| M105 | **closed** | exhaustive residual primitive, applicability, ownership, and security audit; no production behavior |
| M106 | **ready** | six TCP-client `DelayOpen` cells through the existing I2PControl client-listener owner |

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
- `105-residual-tunnel-option-primitive-audit.md`

## M105 — closed audit

M105 is closed. It preserved all 164 production blockers and identified one
bounded successor.

It audits all 164 M104 `blocked_primitive` cells and creates:

`105-residual-option-audit.toml`

The audit must distinguish:

- exact I2PControl-local implementation candidates;
- candidates requiring a minimal neutral existing-owner seam;
- actual Yosemite/SAM/dependency blockers;
- genuine architecture-decision blockers;
- cells whose current applicability may be wrong under the pinned/reference semantics;
- unresolved semantic conflicts.

M105 is evidence-only. It does not change production behavior, M095 support dispositions, Cargo dependencies, `Cargo.lock`, Yosemite, router core, tunnel data planes, frontend code, or workflows.

Each residual record must name exact Proposal/reference semantics, current Emissary owner/blocker, actual Yosemite/SAM wire support where relevant, security/anonymity impact, exact candidate paths where a contained implementation may exist, and one bounded audit disposition.

The closure registered exactly one dependency-ready successor: M106 for six
TCP-style client families. Streamr `DelayOpen` is semantic-blocked, and the
other residual groups remain deferred.

## M106 — current ready implementation

`106-delay-open-client-listener.md` is the sole current ready handoff. It is
confined to the existing I2PControl client-listener owner and must not change
Yosemite, core/util crates, dependencies, Streamr behavior, or the production
support counts until its own closure evidence is complete.

## Residual families under M105

M104 groups the 164 cells as:

- `Shared` — 7;
- `UseSSL` — 4;
- `TunnelVariance`, `TunnelBackupQuantity`, `SigType`, `CustomOptions` — 40;
- `NewDest`, `PersistentClientKey` — 14;
- `PrivKeyFile` — 10;
- `UseOutproxyPlugin`, `SSLProxies`, `JumpList` — 12;
- `ConnectDelay`, `Profile`, `DelayOpen`, `Reduce*`, `Close*` — 56;
- `AllowInternalSSL`, `UniqueLocalAddressPerClient`, `MultiHoming` — 6;
- `EncryptLeaseSet`, `OptionalLookup`, `LeaseSetClientAuths` — 15.

Implementation difficulty alone is not evidence that a cell is `not_applicable`. Conversely, a Java-specific implementation mechanism is not automatically a required Emissary architecture. M105 must separate contract semantics from implementation mechanism.

## Closed M098/M099 corrective slices

M098 closed as a bounded client/proxy/HTTP slice. It applied exact behavior already owned by existing I2PControl runtimes and transferred genuine plugin/TLS-proxy/jump-list/client-management gaps to explicit residual ownership.

M099 closed internally as a bounded server slice. It applied presentation/access/filter/admission/rate behavior through accepted server runtime paths and left LeaseSet/TLS/address-routing gaps blocked where exact semantics are unavailable.

Neither pass authorized Yosemite changes, new core APIs, dependency/lockfile changes, SOCKS protocol expansion, router-wide banning, arbitrary filesystem access, request-selected LAN targets, or weakened anonymity boundaries.

## M104 closure

M104 is closed as blocked. Its closure:

`plans/closure/i2pcontrol-proposal-170/104-closure.md`

records passing bounded local/live evidence but correctly refuses full support because 164 applicable TunnelManager cells remain blocked.

A future M104 reattempt requires:

- zero applicable `blocked_primitive`, `planned_apply`, unsupported, or unknown cells;
- then full live/reseeded/reference-router interoperability and security/containment reclosure.

## Full-support design rules

### AddressBook

M096 is current authority. All 13 keys are operational with typed persistence, path confinement, bounded downloader integration, and metadata-only theme behavior. Do not reopen it for tunnel option work.

### Tunnel options

Every accepted applicable option must change real runtime behavior. Parser acceptance, rawConfig persistence, or fail-before-allocation rejection is not final support.

No audit or implementation plan may convert a blocker into support merely because an approximate behavior exists at a different layer.

### RouterInfo

M100-M103 close the former five source gaps. The final 43-row state is 42 available / 1 protocol-permitted neutral / 0 unavailable.

### ClientServicesInfo

The six-selector implementation remains closed and is regression scope only unless a future reclosure finds a direct defect.

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

Use focused tests plus the existing feature-gated matrix/containment guards. Do not add a CI farm.

For M105, the audit guard should verify only exact 164-cell coverage, unique records, allowed disposition vocabulary, and mandatory evidence fields for candidate/blocker classes.

The historical `m063_feature_reachability` test target is absent in the current checkout; preserve that as a tooling/inventory limitation rather than inventing unrelated scope.

The repository also has a known nightly/stable rustfmt mismatch. Do not create formatter-only churn across audited core files solely to make that gate green.

## Closure discipline

A plan closes only with a closure record containing exact commits, requirement-to-evidence mapping, verification outcomes, security/compatibility/containment review, unresolved findings, and internal-only attestation.

M105 closure does not imply any residual cell is implemented. It decides whether a bounded successor exists and may register at most one next handoff.

Only a successful future M104 reattempt may state full Proposal 170 support against the pinned revision.
