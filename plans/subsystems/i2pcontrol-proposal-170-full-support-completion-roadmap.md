# I2PControl Proposal 170 Full-Support Completion Roadmap

Status: **active / partial; M131 closed as blocked; no successor registered**

Current registered handoff: **none**. M131 found no dependency-ready successor.

Current runtime/security qualification authority:

- `plans/closure/i2pcontrol-proposal-170/130-closure.md`
- M130 implementation head `fe1a981`
- M130 closure / M131 production-behavior baseline `a68094e128d2b92f0fd5b350e38512ef6b65cb6b`

Pinned external authority:

- I2P Proposal 170, `I2PControl Expansion`;
- revision `2026-05-20`;
- status `Open`.

All external specifications, source trees, documentation, issues, pull requests and reference routers are read-only evidence. Repository writes remain internal to `eggstack/emissary`; `eggstack/yosemite` is writable only under its own registered planning ceremony and ADR-0005. No upstream issue/PR/review/contact/release activity is authorized.

## 1. Purpose

Move the internal fork from truthful partial Proposal 170 support toward full support against the pinned revision while keeping Proposal-specific business/admin/application policy under `emissary-cli/src/i2pcontrol/**` wherever possible and preserving Emissary's audited router/core boundaries.

Full support means exact runtime and externally observable semantics. Parser acceptance, persisted configuration, dormant fields, serializer reachability, fake/default state or approximate behavior do not count as support.

This is not a general base-I2PControl parity program, router redesign, plugin framework project or upstream contribution program.

## 2. Canonical/internal authority

Read in this order:

1. `plans/000-long-term-specification.md`;
2. `plans/001-terminology-and-domain-model.md`;
3. `plans/002-long-term-roadmap.md`;
4. `plans/003-planning-process.md`;
5. ADR-0001 through ADR-0005;
6. M061/M062 containment authority;
7. M093 tunnel-security authority;
8. M095 full-support matrix;
9. M105 residual audit;
10. M110 completion ledger;
11. current closure authority M130;
12. this roadmap, the registry and the specific registered handoff.

Historical closure records remain immutable evidence. Later correctives/audits supersede only the claims they explicitly replace.

## 3. Current production/support state

The currently qualified subset includes:

- RouterInfo: 43 Proposal additions / 42 available / 1 protocol-permitted neutral / 0 unavailable;
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and deterministic effective lookup;
- all 12 canonical TunnelManager data planes;
- all seven canonical TunnelManager actions;
- all six ClientServicesInfo selectors;
- API-1 authentication/version/token behavior required by the extension surface;
- HTTPS-only serving, bounded JSON-RPC 2.0 single/batch dispatch, finite token lifetime and fail-closed non-loopback TLS configuration;
- M110/M116 shared-session and destination ownership where currently claimed;
- M118 neutral tunnel variance/backup semantics;
- bounded HTTP/IRC/SOCKS/Streamr application/security behavior already closed by prior milestones.

M130 is the current implemented-subset operational/security qualification authority.

The authoritative M095 matrix at the M131 starting baseline is:

- 70 option rows × 12 tunnel types = 840 cells;
- `284 apply`;
- `88 blocked_primitive`;
- `468 not_applicable`;
- zero planned/unknown/accept-inert cells.

Full Proposal 170 support is **not** claimed.

## 4. Ownership and containment

### 4.1 Preferred owner

Proposal-specific policy belongs under:

- `emissary-cli/src/i2pcontrol/**`.

That includes JSON-RPC validation, TunnelManager administrative policy, durable definitions, application-layer HTTP/IRC/proxy/TLS behavior, I2PControl-owned secret stores, session registries and support evidence.

### 4.2 Neutral lower-layer exception

A future production path outside I2PControl is allowed only when all are true:

1. the behavior belongs to an existing canonical lower-layer owner;
2. no truthful I2PControl-local implementation exists;
3. the exact paths are named before implementation;
4. the seam is neutral rather than Proposal-shaped;
5. unrelated router behavior remains unchanged;
6. containment/dependency evidence is explicitly updated;
7. a registered plan authorizes that exact path budget.

M118 is the historical example: generic SAM tunnel-pool variance/backup behavior, not Proposal policy in core.

### 4.3 Dependency boundary

Yosemite remains the sole accepted SAM implementation for the I2PControl path. The exact Y005 fork revision `59140a2277bf296928d2e8ce39a148182eeff044` is consumed only through the optional `yosemite-i2pcontrol` alias.

No global patch, vendoring, path override, floating fork, parallel raw SAM implementation or automatic Yosemite expansion is authorized.

## 5. Cross-cutting invariants

All future work MUST preserve:

- exact pinned names/types/actions/presence semantics;
- no fabricated or accept-inert state;
- every `apply` cell produces the real runtime effect;
- matrix counts are evidence, not goals;
- unsupported values fail before allocation/effect and never silently downgrade;
- direct I2P proxy traffic never falls through to clearnet DNS;
- clearnet proxying requires explicit accepted I2P outproxy behavior;
- trusted Yosemite-derived remote peer identity;
- literal-loopback/local-target confinement unless a separately accepted neutral primitive proves an equally strong model;
- bounded admission/tasks/timers/state and generation-local cancellation;
- transactional edit/start/restart behavior and last-known-good preservation where promised;
- no lock held across unrelated network/filesystem I/O, sleeps, joins or cancellation waits;
- secret/key/path confinement and redaction;
- no silent LeaseSet-security downgrade;
- feature-disabled/runtime-disabled isolation;
- no unrelated base API parity;
- no frontend coupling;
- external interaction remains read-only/internal-only.

## 6. Current residual partition

M131 starts from exactly 96 blocked cells. The current prose partition is:

- 4 `UseSSL` cells;
- 10 `SigType` cells;
- 63 client proxy/profile/reduction/lifecycle cells, including the 18 TCP `Close`/`CloseTime`/`NewDest` cells demoted by M121 plus Streamr residuals;
- 19 server presentation/routing/LeaseSet cells.

M131 MUST derive the exact starting set mechanically from M095 rather than trusting this grouping.

No blocked cell may become `apply` during M131 because M131 authorizes no runtime capability implementation.

## 7. Why M131 exists

M114 closed as blocked because applicable residual cells remained. M125 then corrected two `AllowInternalSSL` applicability cells, and M127-M130 corrected/requalified the shared control plane without changing the option matrix.

Subsequent residual research found enough evidence that the 96-cell blocker inventory needs a fresh semantic/ownership freeze before implementation:

- `MultiHoming` may be `shouldBundleReplyInfo` behavior rather than host-interface multihoming;
- `Profile=interactive` may be bounded streaming-window behavior rather than router-global profile selection;
- `SSLProxies` and `JumpList` may be HTTP-client-specific rather than generic proxy-family options;
- Streamr applicability requires its own datagram/session review;
- `UseSSL` family applicability/direction requires a fresh exact freeze before any TLS work;
- `UniqueLocalAddressPerClient` may be a confined source-bind policy rather than LAN-routing expansion;
- session `Reduce*` and `Close*` share a real lower-layer activity/lifecycle primitive rather than local TCP-handler count;
- Yosemite Y005 now serializes typed encrypted-LeaseSet/client-auth options, so stale serializer blocker language must move to the first actually missing Emissary runtime owner.

M131 therefore re-freezes applicability, semantics and primitive architecture. It does not implement those primitives.

## 8. M131 — residual applicability and primitive-architecture re-freeze

Plan:

- `plans/implementation/i2pcontrol-proposal-170/131-residual-applicability-and-primitive-architecture-refreeze.md`

Status: **closed as blocked**.

M131 delivered:

1. enumerate every starting blocked cell;
2. re-freeze each cell against Proposal/reference/runtime evidence;
3. permit only `blocked_primitive` retention/correction or evidence-backed `not_applicable` correction;
4. forbid `apply` promotion;
5. produce a machine-readable residual primitive map;
6. identify canonical owners and minimal future path budgets;
7. cluster shared primitives;
8. rank future work by semantic certainty, containment cost, security risk and dependency readiness;
9. assessed future M132+ readiness; no successor was dependency-ready.

M131 was planning/evidence/documentation-only. It authorized no production Rust, Cargo/dependency or Yosemite changes.

Closure authority: `plans/closure/i2pcontrol-proposal-170/131-closure.md`.
The reconciled matrix is `284 apply / 88 blocked_primitive / 468 not_applicable`; eight cells have affirmative applicability corrections and no cell was promoted to `apply`.

## 9. Required M131 semantic lanes

M131 must explicitly resolve the following lanes.

### 9.1 Proxy/outproxy applicability

Re-freeze `UseOutproxyPlugin`, `SSLProxies` and `JumpList` type-by-type. Determine which data planes actually consume them and correct inherited applicability if affirmative reference evidence supports that correction.

### 9.2 Streamr applicability

Review all blocked Streamr client cells independently from TCP assumptions, including `ConnectDelay`, `Profile`, `DelayOpen`, `Reduce*`, `Close*` and `NewDest`.

### 9.3 Streaming profile

Determine whether `Profile=interactive` is exactly a streaming max-window/configuration behavior and locate the smallest neutral streaming owner needed below I2PControl.

### 9.4 Session activity/reduction/close/resume

Freeze the exact application-activity predicate and required neutral control primitives for:

- `Reduce`;
- `ReduceCount`;
- `ReduceTime`;
- `Close`;
- `CloseTime`;
- `NewDest`.

Local TCP socket count is not an acceptable substitute. The candidate architecture must assess session-local application-activity observation, tunnel-pool reduction/restoration and a typed idle-close/resume lifecycle outcome.

### 9.5 Unique local source address

Freeze `UniqueLocalAddressPerClient` source-address derivation, IPv4/IPv6 behavior, socket binding order and confinement impact. Prefer an I2PControl-only source-bind policy if exact behavior can preserve the existing loopback target boundary.

### 9.6 MultiHoming

Verify whether the reference contract is `shouldBundleReplyInfo`. If so, locate the neutral session-state and outbound-message LeaseSet-bundling owners. Do not create a Proposal-shaped core multihoming manager.

### 9.7 UseSSL

Re-freeze exact applicable families and distinguish management HTTPS, SAM-control TLS, local client-facing TLS, local server-target TLS and HTTP SSL-outproxy behavior. Future presentation TLS should remain I2PControl-owned wherever feasible and must be fail-closed with explicit identity/trust ownership.

### 9.8 SigType

Decompose the genuine destination/private-signing work: supported value set, key generation/import, destination key-certificate encoding, streaming/datagram signing and persistent/transient identity compatibility. No crypto dependency is added by M131.

### 9.9 Encrypted/authenticated LeaseSets

Re-base `EncryptLeaseSet`, `OptionalLookup` and `LeaseSetClientAuths` on Yosemite Y005's actual typed wire support and identify the first missing Emissary runtime owner. Separate legacy encrypted LeaseSet behavior from encrypted LS2/blinded/DH/PSK modes when their protocol owners differ.

## 10. Candidate future primitive clusters

The following are hypotheses M131 must test, not pre-authorized milestones:

- HTTP address-helper / SSL-outproxy behavior;
- local presentation TLS;
- streaming profile/window configuration;
- I2P-session activity + idle reduction/close/resume;
- per-client local source addressing;
- `shouldBundleReplyInfo` / sender LeaseSet bundling;
- outproxy-provider/plugin integration;
- destination signing-type generation;
- encrypted/authenticated LeaseSets.

For each retained cluster M131 must record:

- exact cells;
- semantic contract;
- canonical owner;
- path budget;
- dependency edges;
- failure/cancellation/restart/contention model;
- secret/security hazards;
- deterministic test plan;
- reference/live interoperability requirement;
- stop conditions;
- readiness.

## 11. Dependency graph

```text
M114 final reclosure                         [CLOSED AS BLOCKED]
M125 residual capability audit              [CLOSED]
M126 historical requalification             [HISTORICAL]
  |
  +--> M127 token lifetime                  [CLOSED]
        |
        v
      M128 JSON-RPC batch                   [CLOSED]
        |
        v
      M129 remote TLS fail-closed           [CLOSED]
        |
        v
      M130 integrated requalification       [CLOSED — CURRENT RUNTIME AUTHORITY]
        |
        v
      M131 residual semantic/primitive map  [CLOSED AS BLOCKED]
        |
        +--> no dependency-ready M132+ handoff; retained clusters remain unregistered
```

No M132+ capability plan is currently registered.

## 12. Historical milestone authority

Important closed milestones remain authoritative for their bounded claims:

| Milestone | Current role |
|---|---|
| M061/M062 | containment/dependency authority |
| M093 | tunnel application/security boundary |
| M095 | machine-readable support matrix |
| M105 | residual-option audit |
| M110/M116 | shared-session/destination ownership and corrective authority |
| M111 | session-wire applied subset; historical UseSSL blocker |
| M112 | client proxy/lifecycle historical closure |
| M113 | server residual historical closure |
| M117 | exact optional Yosemite dependency seam |
| M118 | neutral variance/backup tunnel-pool behavior |
| M121 | SigType and Close/CloseTime/NewDest semantic truthfulness demotion |
| M122/M124 | exact Yosemite Y004/Y005 transport adoption |
| M123 | cancellation/commit atomicity corrective |
| M125 | AllowInternalSSL applicability correction and residual audit |
| M127 | finite token lifetime |
| M128 | bounded JSON-RPC batch conformance |
| M129 | non-loopback managed-TLS fail-closed |
| M130 | current implemented-subset runtime/security qualification |

M131 does not rewrite those historical closures.

## 13. Failure, restart and security requirements for successor readiness

A future primitive is not dependency-ready unless its architecture defines:

- validation-before-effect boundary;
- allocation/publication point;
- cancellation owner;
- generation/restart ownership;
- bounded task/timer/state behavior;
- lock scope;
- rollback/old-generation preservation;
- shared-session contention behavior where applicable;
- secret lifetime/redaction/zeroization requirements;
- lower-layer partial-failure behavior;
- no silent security downgrade or fallback.

A simple happy path with no credible failure model remains blocked.

## 14. Planning/registration discipline

Per `plans/003-planning-process.md`:

- M131 was the final registered Proposal-170 evidence handoff and is now closed;
- future M132+ plans may be described by the M131 artifact but remain unregistered until dependencies are known;
- M131 closure registered no next dependency-ready handoff;
- material deviations require plan/ADR correction before production code;
- closure evidence, not implementation assertions, decides completion;
- matrix-count reduction is never itself an acceptance criterion.

## 15. Verification policy

M131 is planning/evidence-only. If it changes M095/M105/M110 or focused matrix guards, run the relevant suites plus containment checks. Baseline commands are:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --no-fail-fast
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

If Rust files change only to enforce matrix guards, `cargo fmt --all -- --check` should also be attempted; pre-existing stable/nightly formatter drift is recorded rather than normalized through unrelated churn.

## 16. Risks and deferred decisions

### Reference disagreement

The Proposal, unmerged Java Proposal implementation and current I2PTunnel runtime may disagree. M131 must record the conflict and retain the safer blocked disposition unless the repository's authority hierarchy resolves it. It must not select the interpretation that merely lowers the blocked count.

### Crypto scope

`SigType` and encrypted LeaseSets may require genuine lower-layer cryptography and destination/NetDB changes. Those are separate neutral protocol projects with interoperability gates, not small I2PControl parity patches.

### Plugin parity

If `UseOutproxyPlugin` requires a real registered provider/plugin subsystem and Emissary has no independent architectural need for one, it may remain blocked indefinitely rather than creating broad machinery solely for Proposal parity.

### TLS identity/trust

Any future `UseSSL` implementation must have explicit identity/trust ownership and must not reuse management TLS accidentally or disable peer verification for convenience.

## 17. Exit conditions

M131 closes when:

- all 96 starting blocked cells are individually accounted for;
- final applicability is evidence-backed;
- zero cells are promoted to `apply`;
- stale blocker descriptions are corrected to the first real missing runtime primitive;
- a machine-readable residual primitive map exists;
- future path budgets/dependency edges/security models are explicit;
- M095/M105/M110/docs are reconciled if current authority changes;
- no production/dependency/Yosemite code changed;
- at most one next dependency-ready M132+ handoff is registered.

The full-support roadmap itself closes only after a future successful final reclosure proves zero applicable residual gaps, live/reference interoperability, no high/medium Proposal-scoped security/correctness defect and explained minimal lower-layer seams.

Until then, official status remains **partial Proposal 170 support**.
