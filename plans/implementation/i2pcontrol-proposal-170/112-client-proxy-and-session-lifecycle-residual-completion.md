# M112 — Client Proxy and Session-Lifecycle Residual Completion

Status: **closed as blocked** — six TCP client families now apply `ConnectDelay`, `Close`, `CloseTime`, and `NewDest`; 45 M112-owned residual cells remain explicitly blocked

Class: capability / client runtime lifecycle / proxy policy

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Source evidence:

- M098 closure: `plans/closure/i2pcontrol-proposal-170/098-closure.md`
- M105 audit: `plans/implementation/i2pcontrol-proposal-170/105-residual-option-audit.toml`
- M106 closure: `plans/closure/i2pcontrol-proposal-170/106-closure.md`
- M093 security authority.

Pinned authority: I2P Proposal 170 revision `2026-05-20`, status Open.

Closure record: `plans/closure/i2pcontrol-proposal-170/112-closure.md`

External references are read-only. Repository writes are internal-only.

## 1. Objective

Resolve the remaining client/application residual option families after M106:

- `UseOutproxyPlugin`, `SSLProxies`, `JumpList` — 12 M105 cells;
- `ConnectDelay`, `Profile`, remaining `DelayOpen`, `Reduce*`, and `Close*` lifecycle rows — 50 current cells after M106 moved six TCP-client `DelayOpen` cells to `apply`.

The current M112 inventory is therefore 69 blocked cells (the original 62 plus seven `NewDest` cells transferred by M116), subject to an exact baseline freeze from M095/M105 at execution time.

M112 must distinguish Proposal 170 contract effects from Java I2PTunnel implementation mechanisms. A Java plugin class, profile object, or timer implementation is not automatically something Emissary must recreate. Cells may move from `blocked_primitive` to `not_applicable` only with affirmative pinned/reference evidence; implementation difficulty is never sufficient.

## 2. Hard blockers and readiness

Before execution, M112 required registration until:

1. M110/M111 establish the final client session ownership/configuration object that lifecycle options would act upon;
2. the current M095/M105 residual inventory is re-frozen;
3. every candidate row has one of: an exact existing I2PControl runtime owner, a specifically authorized neutral existing-owner seam, or direct evidence that the Java-specific mechanism is not part of the cross-router Proposal contract.

The plan does not authorize creating a generic plugin framework, TLS interception framework, router profile subsystem, or global timer service.

## 3. Invariants

M112 MUST preserve:

- direct `.i2p` proxy traffic never falls through to clearnet DNS;
- clearnet proxy traffic requires an explicit I2P outproxy;
- no request-selected LAN/local target expansion;
- proxy authentication/secrets remain redacted;
- lifecycle timers are generation-local, bounded, monotonic and cancellable;
- no sleeping task survives its tunnel generation;
- no timer callback may mutate a newer edited/restarted generation;
- no lock crosses network I/O or sleeps;
- `DelayOpen` retains M106 lazy-allocation semantics for the six implemented TCP client families;
- Streamr remains datagram-specific and is never forced through TCP listener abstractions;
- M110 shared-session membership and M111 session-wire settings are respected by reduce/close/restart behavior;
- no plugin/module loading from request-controlled paths;
- no frontend or router-global profile ownership;
- feature-disabled/default behavior remains unchanged.

## 4. Explicit non-goals

M112 MUST NOT:

- build Java I2PTunnel's plugin framework;
- implement generic dynamic library/plugin loading;
- add transparent TLS MITM/certificate generation for proxies;
- add SOCKS BIND/UDP ASSOCIATE, DCC/WEBIRC, or other non-pinned protocol features;
- create a router-wide peer/tunnel profile subsystem to satisfy a configuration name;
- reimplement SAM or modify Yosemite;
- add core/router APIs solely for timer convenience;
- implement server/LeaseSet residuals from M113;
- weaken M093 HTTP/IRC/proxy security boundaries;
- initiate upstream activity.

## 5. Expected production paths

Preferred scope is:

- `emissary-cli/src/i2pcontrol/domain/tunnel.rs`;
- `emissary-cli/src/i2pcontrol/backends/options.rs`;
- affected client backends (`client`, `http_client`, `irc_client`, `socks`, `socks_irc`, `connect_client`, `streamr` as applicable);
- shared runtime helpers under `emissary-cli/src/i2pcontrol/backends/runtime/**`;
- `emissary-cli/src/i2pcontrol/tunnel_manager.rs`;
- M095/M105 artifacts and focused tests.

Any production change outside I2PControl requires a separately registered exact-path amendment before implementation. No core/util/dependency change is pre-authorized.

## 6. Work packages

### WP1 — Revalidate applicability by contract effect

For each of the 69 current cells, record:

- pinned Proposal type/value;
- reference behavior visible to an I2PControl client;
- whether the behavior is a contract effect or only a Java implementation mechanism;
- exact current Emissary owner;
- security impact;
- final pre-implementation disposition.

This is not permission to reduce the blocked count by interpretation convenience. `not_applicable` requires positive evidence.

### WP2 — `ConnectDelay`

If the pinned behavior is a bounded delay before remote connection/session use, implement it in the per-tunnel generation owner using monotonic cancellable sleep.

- validate a finite bounded duration;
- do not hold listener/session locks during the delay;
- cancellation/edit/restart invalidates the timer;
- lazy session creation from M106 and connection delay must have one deterministic ordering.

### WP3 — `Reduce*` / `Close*`

Implement only where the existing I2PControl runtime can observe the required condition truthfully.

Candidate design:

- one generation-local policy state object;
- bounded monotonic timers/counters;
- explicit transition to reduced tunnel/session settings only if the underlying session owner exposes a real reconfiguration/restart mechanism;
- close-on-idle/time/count semantics must cancel/stop the intended generation, not simply toggle metadata;
- shared sessions from M110 must not be reduced/closed by one member in a way that violates another member's contract.

If Yosemite cannot modify the required tunnel/session property and a restart would change identity/semantics incorrectly, keep that cell blocked rather than approximating.

### WP4 — remaining `DelayOpen`

Re-evaluate only the Streamr/datagram or other post-M106 cell(s). Do not reuse the TCP listener implementation mechanically.

Implement only if the pinned semantic has a meaningful datagram equivalent and there is a contained runtime owner. Otherwise retain `semantic_blocked` with direct evidence.

### WP5 — `Profile`

Determine whether Proposal 170 requires selection of a portable externally visible behavior or merely a Java I2PTunnel profile implementation.

- if portable semantics map to an existing typed Emissary session policy, implement that mapping explicitly;
- if it names an absent Java profile subsystem without a cross-router contract effect, reclassify only with affirmative evidence;
- do not add a generic profile registry for parity.

### WP6 — `UseOutproxyPlugin`, `SSLProxies`, `JumpList`

Separate contract from Java mechanism.

- outproxy routing must remain explicit I2P routing with no clearnet fallback;
- if `UseOutproxyPlugin` merely selects Java's outproxy provider implementation and Emissary has no plugin concept, document applicability/reference evidence rather than inventing plugins;
- if `SSLProxies` means a bounded list of destinations/proxies with distinct routing behavior, use existing outproxy selection policy only when exact semantics are supportable;
- `JumpList` must not cause arbitrary URL/host fetching, DNS resolution, or open redirect behavior unless the pinned contract explicitly requires and a safe I2P-only owner exists.

Unknown or unsafe values fail before allocation.

### WP7 — Matrix and runtime evidence

Every `apply` transition requires request→runtime evidence. Every `not_applicable` transition requires pinned/reference evidence captured in M105/M095 comments/tests. No `accept_inert` disposition may appear.

## 7. Failure, cancellation, restart, contention

Timers/workers are owned by tunnel generation IDs and are aborted/released on stop/delete/edit/restart. Timer callbacks must compare generation identity before taking action.

No unbounded per-request timers, queues, jump lists, proxy lists, or state maps. Apply existing inventory/request bounds or tighter explicit limits.

A failed policy transition preserves the previous truthful running state or reports the tunnel failed/stopped; it must not claim a reduction/close policy was applied when the session could not enact it.

## 8. Compatibility and migration

Previously blocked options remain non-operational until explicitly implemented. Existing stored definitions must pass new validation before activation. No automatic behavior is inferred from unknown raw fields.

No router.toml, proxy-global configuration, or frontend migration.

## 9. Focused tests

At minimum:

- cancellable `ConnectDelay` with edit/restart generation replacement;
- idle/count/time close/reduce transitions where implemented;
- shared-session safety under one member's lifecycle policy;
- no timer/task leak after repeated restart/edit;
- Streamr DelayOpen disposition/evidence;
- proxy/plugin/jump options cannot trigger local DNS, direct clearnet fallback, arbitrary filesystem/plugin loading, or unsafe URL targets;
- exact applicability reclassifications are guarded by matrix/audit tests;
- blocked options continue to fail before allocation.

## 10. Broad verification

Run the standard feature, containment, M095/M105, live-runtime, check, clippy, fmt-attempt and diff gates. Re-run focused HTTP/SOCKS/IRC/Streamr security suites for every affected backend.

No new hosted CI/fuzz/release system.

## 11. Acceptance criteria

M112 closes only when:

1. all targeted cells are `apply`, evidenced `not_applicable`, or explicitly remain blocked with an exact primitive/semantic reason;
2. no Java-specific framework was recreated without contract need;
3. every applied lifecycle option changes real generation behavior and is cancellable/bounded;
4. proxy changes preserve no-DNS/no-clearnet-fallback and existing security filters;
5. no timer/task/state leak or cross-generation mutation exists;
6. M095/M105 exact counts and deltas are updated truthfully;
7. M061/M062/M093 evidence remains green;
8. closure names the next registry-ready handoff.

## 12. Stop conditions

Stop a sub-slice if:

- exact semantics require Yosemite/session capabilities absent after M111;
- implementation would need a plugin framework, TLS MITM stack, router-global profile service, or broad core timer owner;
- Streamr semantics are not meaningfully defined by the pinned contract;
- an option would weaken proxy anonymity/trust boundaries;
- applicability cannot be established affirmatively.

## 13. Closure evidence

Require exact cell ledger, runtime/timer/proxy tests, cancellation/contention review, security/no-DNS evidence, matrix/audit deltas, full verification outcomes, unresolved findings, next-handoff decision, and internal-only attestation.

## 14. Internal-only boundary

No upstream issue/PR/review/submission/merge/adoption request, plugin/dependency contribution preparation, branch/tag push, release, or maintainer contact is authorized. External references are read-only.

## 15. Closure disposition

M112 completed its authorized portable client-lifecycle slice and closed the
remaining rows with exact dispositions. `ConnectDelay`, `Close`, `CloseTime`, and
`NewDest` apply for `client`, `httpclient`, `ircclient`, `socks`, `socksirc`, and
`connectclient`. Proxy/plugin/TLS-jump behavior, `Profile`, `Reduce*`, and the
Streamr lifecycle cells remain blocked because no safe accepted owner exists.
The matrix delta is `288 / 94 / 458` to `312 / 70 / 458`; M113 and M114 remain
blocked and no future plan becomes dependency-ready from this closure.
