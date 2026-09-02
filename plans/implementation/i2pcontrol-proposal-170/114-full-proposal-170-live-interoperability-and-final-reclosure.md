# M114 — Full Proposal 170 Live Interoperability and Final Reclosure

Status: **closed as blocked** — final reclosure evidence gathered at the current head; the hard gate remains false because 70 applicable residual cells are still blocked

Class: capability closure / interoperability / security reclosure

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Predecessor authority:

- M104 prior blocked reclosure: `plans/closure/i2pcontrol-proposal-170/104-closure.md`
- M109–M113 plans and closures when available;
- M093 tunnel security closure;
- M095 full-support matrix;
- M105 residual audit.

Pinned authority: I2P Proposal 170 revision `2026-05-20`, status Open.

External specifications/reference implementations are read-only evidence. All repository writes remain internal to `eggstack/emissary`.

## 1. Objective

Perform the final independent full-support reclosure after all implementation blockers are resolved. M114 introduces no new feature merely to make closure pass. It proves that the current fork implements the pinned Proposal 170 contract operationally, securely, and with the accepted containment boundary.

M114 must answer, with direct current-head evidence:

1. Are all Proposal 170 methods/selectors/actions/types and applicable options wire-correct?
2. Does every claimed capability reach a real production owner rather than a fake/inert source?
3. Are the twelve tunnel families operational with their applicable option semantics?
4. Do startup-visible and control-plane-created tunnel action semantics behave consistently with the pinned contract?
5. Does enabled AddressBook state remain coherent with normal resolver behavior and confined persistence?
6. Are RouterInfo and ClientServicesInfo truthful under live network conditions?
7. Do local and reference-router interoperability tests pass without weakening security/anonymity boundaries?
8. Are changes outside `emissary-cli/src/i2pcontrol/**` still minimal, neutral, and explicitly justified?

Only a successful M114 closure may change the workstream status to full Proposal 170 support against the pinned revision.

## 2. Hard readiness gate

M114 MUST NOT be registered ready until all are true:

- M109 closed;
- M110 closed or all its target cells are evidence-backed `not_applicable` with no remaining blocker;
- M111 closed or all its target cells are evidence-backed `not_applicable` with no remaining blocker;
- M112 closed or all its target cells are evidence-backed `not_applicable` with no remaining blocker;
- M113 closed or all its target cells are evidence-backed `not_applicable` with no remaining blocker;
- M095 TunnelManager matrix has zero `blocked_primitive`, `planned_apply`, unsupported, unknown, or accept-inert applicable cells;
- M105 residual audit is reconciled to zero unresolved applicable residuals;
- no open high/medium Proposal-170-scoped security corrective exists.

If any gate is false, M114 stops before interoperability certification and remains blocked.

## 3. Invariants

M114 MUST preserve and re-verify:

- exact pinned names/casing/types/presence semantics;
- API version 1-only authentication behavior;
- HTTPS-only listener and M107/M108 managed-TLS protections;
- constant-time bounded authentication and bounded token/throttle state;
- no fabricated RouterInfo or service state;
- one enabled AddressBook runtime authority with cross-book precedence and confined SetConfig paths;
- no runtime/default feature contamination when I2PControl is disabled;
- exact 12 tunnel types and seven canonical actions;
- every applicable option has a real runtime effect;
- no direct I2P→clearnet DNS fallback;
- explicit I2P outproxy requirement for clearnet proxy traffic;
- literal-loopback/local-target confinement unless a separately accepted later security decision explicitly changed it;
- trusted Yosemite-derived remote identity;
- bounded transactional server admission;
- HTTP/IRC/Streamr bounds and filtering;
- secret-safe persistence/logging/RPC behavior;
- no LeaseSet security downgrade;
- generation-local cancellation/edit/restart behavior;
- minimal neutral non-I2PControl seams only;
- no frontend coupling;
- internal-only external interaction.

## 4. Explicit non-goals

M114 MUST NOT:

- implement a feature that should have been owned by M109–M113;
- change matrix applicability simply to pass the gate;
- add unrelated base I2PControl methods/API 2;
- redesign tunnel data planes;
- add hosted CI farms, long-running fuzz infrastructure, release automation, or benchmark programs;
- weaken security to match a reference implementation;
- submit work upstream or request external review/adoption.

A discovered implementation defect requires a new numbered corrective, not an opportunistic M114 production patch except for trivial planning/test bookkeeping explicitly permitted by the closure plan.

## 5. Work packages

### WP1 — Freeze and verify the canonical inventory

At the reviewed head, mechanically assert:

- Proposal 170 method inventory and compatibility aliases are separated;
- RouterInfo has the exact 43 pinned additions with truthful availability/neutral dispositions;
- AddressBook has exact books, canonical parameter modes, subscriptions and all 13 SetConfig keys;
- TunnelManager has exact 12 types, seven canonical actions and the current 70-row option inventory;
- every 840 option/type cell is `apply` or evidence-backed `not_applicable`;
- ClientServicesInfo has exactly six selectors.

Record hashes of M095/M105 artifacts in closure.

### WP2 — Wire/golden conformance

Run literal JSON fixtures for successful and failing forms:

- Authenticate API 1 / API 2 rejection;
- token placement/conflict/missing/invalid handling;
- JSON-RPC IDs and notifications;
- canonical AddressBook modes including cross-book shadowing, Delete-by-presence, SetSubscriptions and SetConfig;
- canonical lowercase TunnelManager actions, `All`, all types and representative options from each ownership family;
- RouterInfo presence/value type semantics;
- ClientServicesInfo selector presence;
- unsupported base methods remain clearly outside this Proposal-only support claim.

Compatibility aliases must not alter canonical responses.

### WP3 — Local production-composition runtime

Extend/re-run the existing feature-enabled HTTPS live fixture with real production adapters, not fakes.

Required coverage:

- managed TLS/auth;
- AddressBook mutation/restart/path failure;
- RouterInfo request-time sources;
- ClientServicesInfo actual listener state;
- startup + control-plane mixed TunnelManager inventory/actions;
- at least one start/edit/restart/stop/restart-persistence path for each meaningful backend group;
- option families introduced by M109–M113 with real allocation behavior;
- failure rollback and bounded shutdown.

Keep this a bounded integration suite; do not construct a permanent test orchestration service.

### WP4 — Twelve-family traffic evidence

For each canonical tunnel type, obtain direct traffic/data-plane evidence under the current head or cite a current-head regression that exercises the same production path:

- `client`;
- `httpclient`;
- `ircclient`;
- `socks`;
- `socksirc`;
- `connectclient`;
- `streamrclient`;
- `server`;
- `httpserver`;
- `httpbidirserver`;
- `ircserver`;
- `streamrserver`.

Do not infer operational coverage from backend registration alone.

### WP5 — Reference-router interoperability

Run a bounded disposable interoperability matrix against available reference implementations without modifying them.

Preferred evidence:

- Java I2P and/or i2pd as read-only external counterpart(s) in disposable local/VM/container environments;
- real SAM streaming/datagram interaction for affected tunnel families;
- persistent destination/key/session behavior where applicable;
- encrypted/authenticated LeaseSet behavior if implemented by M113;
- HTTP/SOCKS/IRC behavior through I2P paths;
- Streamr datagram subscribe/refresh/fanout behavior.

No external repository write, issue, PR, config submission, or maintainer interaction. Interoperability setup is test infrastructure only and must not become a production dependency.

If the environment cannot run a specific counterpart, record the exact operational limitation; do not claim that implementation's interoperability from protocol-unit tests alone.

### WP6 — Public/reseeded network truthfulness

Where safe and available, run a bounded real/reseeded Emissary instance long enough to obtain nontrivial RouterInfo live values and prove:

- connected peer/tunnel/netdb fields come from live owners;
- transit 15-second sampler behaves under real traffic if transit occurs;
- network error/firewall fields reflect real observation semantics;
- router news source verifies/authenticates as designed;
- banned-peer map remains truthful for the router's actual capability.

Do not make public-network success a reason to weaken local deterministic tests.

### WP7 — Independent security reclosure

Re-audit the final head against M093 plus M107/M108 and all later closures.

At minimum review:

- authentication/TLS/token exposure;
- filesystem/secret stores;
- server target routing/SSRF/DNS fallback;
- trusted peer identity;
- admission/cardinality/task bounds;
- HTTP framing/spoof/privacy;
- IRC registration/DCC/CTCP/lifetime;
- Streamr subscriber/payload/fanout/Sybil residual;
- shared-session isolation;
- key import/persistence;
- lifecycle timer/task cancellation;
- LeaseSet no-downgrade behavior;
- startup lifecycle generation ownership.

Classify every residual explicitly by severity and whether it blocks full support.

### WP8 — Containment/diff audit

Compare the final Proposal 170 production delta to the accepted fork baseline and categorize every path outside `emissary-cli/src/i2pcontrol/**`:

- neutral existing-owner seam;
- application composition wiring;
- tests/docs/planning;
- dependency change explicitly accepted by a prior closure.

Any unexplained or Proposal-shaped core/util change blocks closure.

## 6. Failure/recovery/contention closure

M114 must gather current-head evidence for:

- atomic durable stores and last-known-good recovery;
- failed create/edit/start preserving truthful state;
- bounded stop/restart and no duplicate generation;
- no lock across network I/O/sleeps/joins;
- shared-session teardown/refcount correctness;
- generation-local timer cancellation;
- AddressBook mutation serialization;
- managed TLS fail-closed startup;
- secret-store symlink/type/permission failures;
- all-target lifecycle partial failure behavior.

## 7. Verification commands

At minimum run:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check
cargo test -p emissary-core
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Add only focused commands required by M109–M113/current security suites. Record all outcomes, including known toolchain/flaky issues. Do not convert a failed broad gate into a pass because an isolated rerun succeeds; record both.

Reference-router/public-network commands and environment versions belong in closure evidence, not permanent CI unless separately requested.

## 8. Documentation/static guards

Before closure:

- active docs must not describe implemented cells as blocked or blocked cells as supported;
- support/conformance docs must distinguish Proposal 170 support from unrelated base I2PControl methods;
- M061/M062/M095/M105 guards must match actual allowed paths/counts;
- stale historical planning claims remain historical, not active authority;
- the registry must identify M114 as closing only after the readiness gate is true.

Do not rewrite historical closure records to manufacture consistency.

## 9. Acceptance criteria

M114 closes successfully only when all are true:

1. M095 has zero applicable blocked/planned/unsupported/unknown/accept-inert cells;
2. every canonical Proposal 170 action/type/selector/key is wire-correct and reaches a real owner where applicable;
3. startup/control-plane lifecycle semantics satisfy the pinned contract;
4. all twelve tunnel families have current-head operational evidence;
5. AddressBook, RouterInfo and ClientServicesInfo are truthful under production composition;
6. bounded reference-router interoperability evidence exists for the relevant protocol/tunnel surfaces;
7. no high/medium Proposal-170-scoped security defect remains open;
8. no silent LeaseSet/security downgrade exists;
9. containment review explains every non-I2PControl production path and finds no unjustified expansion;
10. feature-disabled/default Emissary remains unaffected;
11. exact verification outcomes are recorded;
12. closure attests external sources/references were read-only and no upstream interaction occurred.

Only then may docs/registry/roadmap state: **full Proposal 170 support against pinned revision 2026-05-20**. Because Proposal 170 itself remains Open, this is an internal pinned-revision claim, not upstream certification.

## 10. Stop conditions

Stop and open a new numbered corrective if:

- any applicable M095 cell remains blocked/planned/unsupported/unknown/inert;
- a canonical action is only partial for a visible inventory without evidence that the restriction is contract-valid;
- a live/reference test finds a wire/runtime mismatch;
- an implementation claim depends on a fake/zero/default source;
- a high/medium security/anonymity issue is found;
- containment reveals an unexplained core/util/dependency expansion;
- reference interoperability requires changing the reference implementation or contacting upstream;
- full support would require implementing unrelated base I2PControl methods.

## 11. Closure evidence required

The M114 closure must include:

- exact reviewed implementation head and dependency versions;
- requirement-to-evidence matrix for the full Proposal 170 inventory;
- final M095/M105 hashes/counts;
- local live and reference-router environment/topology/results;
- twelve-family operational evidence table;
- RouterInfo/AddressBook/ClientServicesInfo production evidence;
- full security/anonymity review;
- failure/recovery/contention evidence;
- exact changed-path containment classification;
- every verification command and outcome;
- unresolved findings/severity;
- final disposition;
- internal-only external-interaction attestation.

## 12. Internal-only boundary

No upstream issue, pull request, review request, submission, merge/adoption request, branch/tag push, release, contribution package, or maintainer contact is authorized. External specifications, source trees and reference routers are read-only/interoperability evidence only.
