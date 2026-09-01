# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 production support; all twelve tunnel runtimes real; M095-M096 and M098-M103 closed, M099 closed internally/partial, M097 and M104 closed as blocked, M105-M106 closed; **M107 ready**; 224 apply / 158 blocked / 458 not-applicable TunnelManager cells remain

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
- AddressBook CRUD, subscriptions, and all 13 SetConfig keys are operational under the confined owner; M107 is registered to correct cross-book shadowing semantics without reopening M096 confinement.
- Exactly 12 Proposal 170 tunnel types have real backends.
- Exactly 7 canonical TunnelManager actions are implemented.
- M097 applied the supported `TunnelLength`, `TunnelQuantity`, and typed `EncType` semantics.
- M098 applied the bounded client proxy/outproxy/auth/privacy subset.
- M099 applied the bounded server presentation/access/filter/admission/rate subset.
- Remaining primitive-dependent option cells fail before allocation rather than being ignored or reported as applied.
- All 6 ClientServicesInfo selectors are implemented.
- M107 is a corrective control-plane pass for API-version negotiation, AddressBook shadowing, and managed TLS; it does not change TunnelManager support counts.
- Full live/reseeded/reference-router certification remains open.

The current M095 matrix after M106 contains:

- 224 `apply`;
- 158 applicable `blocked_primitive`;
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
| M104 | closed as blocked | final live interoperability/security/containment/full-support reclosure stopped by residual option blockers |
| M105 | closed | exhaustive residual primitive, applicability, ownership, and security audit; no production behavior |
| M106 | closed | six TCP-client `DelayOpen` cells through the existing I2PControl client-listener owner; closure recorded |
| M107 | **ready** | post-M106 corrective pass: API 1-only negotiation, AddressBook cross-book shadowing, managed TLS key/SAN hardening; no matrix change |

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
- `106-delay-open-client-listener.md`
- `107-i2pcontrol-conformance-and-managed-tls-corrective-pass.md`

## M107 — ready corrective implementation

`107-i2pcontrol-conformance-and-managed-tls-corrective-pass.md` is the sole current implementation handoff.

It is based on the post-M106 review against the pinned Proposal 170 text, current API 1 documentation, rejected API 2 proposal, and I2P naming semantics. It corrects exactly three bounded defects already owned by I2PControl:

- API `2` must no longer authenticate; API `1` is the sole supported I2PControl version;
- valid duplicate hostnames in different AddressBook types must be representable with deterministic existing runtime precedence instead of being rejected globally;
- managed TLS private-key publication must be restrictive/fail-closed and newly generated managed certificates must validate for `localhost`, `127.0.0.1`, and `::1`.

M107 does not implement unrelated base methods, invent token-expiration policy, relax AddressBook path confinement, add dependencies, change core/Yosemite/SAM behavior, or change the M095 `224 / 158 / 458` counts.

## M105 — closed audit

M105 is closed. It preserved all 164 production blockers and identified one bounded successor.

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

The closure registered exactly one dependency-ready successor: M106 for six TCP-style client families. Streamr `DelayOpen` is semantic-blocked, and the other residual groups remain deferred.

## M106 — closed implementation

`106-delay-open-client-listener.md` is closed by
`plans/closure/i2pcontrol-proposal-170/106-closure.md`. It implemented the
bounded lazy session owner for the six TCP-client families through the existing
I2PControl client-listener owner. Streamr remains excluded and all other
residual options remain blocked or deferred.

## Residual families under M105

The post-M106 residual is 158 cells. M104's pre-M106 grouped baseline was:

- `Shared` — 7;
- `UseSSL` — 4;
- `TunnelVariance`, `TunnelBackupQuantity`, `SigType`, `CustomOptions` — 40;
- `NewDest`, `PersistentClientKey` — 14;
- `PrivKeyFile` — 10;
- `UseOutproxyPlugin`, `SSLProxies`, `JumpList` — 12;
- `ConnectDelay`, `Profile`, `DelayOpen`, `Reduce*`, `Close*` — 56 before six TCP-client `DelayOpen` cells were applied by M106;
- `AllowInternalSSL`, `UniqueLocalAddressPerClient`, `MultiHoming` — 6;
- `EncryptLeaseSet`, `OptionalLookup`, `LeaseSetClientAuths` — 15.

Implementation difficulty alone is not evidence that a cell is `not_applicable`. Conversely, a Java-specific implementation mechanism is not automatically a required Emissary architecture. M105 separates contract semantics from implementation mechanism.

## Closed M098/M099 corrective slices

M098 closed as a bounded client/proxy/HTTP slice. It applied exact behavior already owned by existing I2PControl runtimes and transferred genuine plugin/TLS-proxy/jump-list/client-management gaps to explicit residual ownership.

M099 closed internally as a bounded server slice. It applied presentation/access/filter/admission/rate behavior through accepted server runtime paths and left LeaseSet/TLS/address-routing gaps blocked where exact semantics are unavailable.

Neither pass authorized Yosemite changes, new core APIs, dependency/lockfile changes, SOCKS protocol expansion, router-wide banning, arbitrary filesystem access, request-selected LAN targets, or weakened anonymity boundaries.

## M104 closure

M104 is closed as blocked. Its closure:

`plans/closure/i2pcontrol-proposal-170/104-closure.md`

records passing bounded local/live evidence but correctly refuses full support because applicable TunnelManager blockers remain.

A future M104 reattempt requires:

- zero applicable `blocked_primitive`, `planned_apply`, unsupported, or unknown cells;
- then full live/reseeded/reference-router interoperability and security/containment reclosure.

M107 correctness fixes do not satisfy or bypass that residual gate.

## Full-support design rules

### AddressBook

M096 remains the base authority for all 13 SetConfig keys, typed persistence, path confinement, bounded downloader integration, and metadata-only theme behavior. M107 reopens only the incorrect cross-book global hostname collision rule and must preserve M096 transactionality and confinement.

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
- I2PControl managed private-key material is not permitted to become a new local confidentiality weakness;
- M088 lower-layer residual is not reopened absent new evidence.

## Verification policy

Use focused tests plus the existing feature-gated matrix/containment guards. Do not add a CI farm.

For M107, use the focused authentication, AddressBook shadowing/persistence, and TLS permission/SAN/symlink tests named in the plan plus the existing feature-gated suite, local live-runtime test, M061/M062 containment tests, check, and clippy.

The historical `m063_feature_reachability` test target is absent in the current checkout; preserve that as a tooling/inventory limitation rather than inventing unrelated scope.

The repository also has a known nightly/stable rustfmt mismatch. Do not create formatter-only churn across audited core files solely to make that gate green.

## Closure discipline

A plan closes only with a closure record containing exact commits, requirement-to-evidence mapping, verification outcomes, security/compatibility/containment review, unresolved findings, and internal-only attestation.

M107 closure must explicitly confirm that the M095 production matrix remains `224 apply / 158 blocked / 458 not-applicable` and that it does not create a new residual-option successor by implication.

Only a successful future M104 reattempt may state full Proposal 170 support against the pinned revision.
