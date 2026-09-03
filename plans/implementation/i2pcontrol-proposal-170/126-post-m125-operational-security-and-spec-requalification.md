# M126 — Post-M125 Proposal 170 Operational, Security, and Spec Requalification

Status: **ready**

Class: corrective requalification / conformance / security / containment

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md`

Predecessor authority:

- M125 closure: `plans/closure/i2pcontrol-proposal-170/125-closure.md`;
- M093 tunnel security closure;
- M095 full-support matrix;
- M105 residual option audit;
- M107/M108 I2PControl conformance and managed-TLS closures;
- M119-M125 corrective closures.

Planning baseline: `685eeeb20f22cdd234e4649c730000d623ad4891`.

Pinned authority:

- I2P Proposal 170 revision `2026-05-20`, status Open;
- base I2PControl specification/API-1 authentication and JSON-RPC behavior.

Current authoritative matrix entering M126: `284 apply / 96 blocked_primitive / 460 not_applicable`.

External specifications and reference implementations are read-only evidence. Repository writes are limited to `eggstack/emissary` and, only under its own separately registered work, `eggstack/yosemite`.

## 1. Objective

Independently requalify the current post-M125 Emissary fork against the pinned Proposal 170 contract and the accepted I2PControl security/containment architecture before any further implementation claim is made.

M126 is deliberately evidence-first. It does **not** assume that a previously closed milestone proves the current head operational, nor does it assume that the 96 blocked cells have become implementable merely because their wire vocabulary is parseable. It must establish, with current-head source tracing and executable evidence, whether every currently claimed `apply` capability:

1. is wire-correct against the pinned proposal;
2. crosses the required authentication/TLS boundary;
3. reaches a real production owner rather than a fake, inert, metadata-only or shadow-state implementation;
4. reports success only after the authoritative state transition is committed;
5. preserves the security, resource and cancellation invariants established by prior corrective milestones;
6. remains contained primarily under `emissary-cli/src/i2pcontrol/**` with only accepted neutral seams outside it.

M126 must also revalidate every `blocked_primitive` disposition and determine whether any blocker has genuinely become dependency-ready. A serializer, parser, accepted option name or test fake is not capability evidence.

The output is one of three truthful dispositions:

- the implemented subset is operationally/security qualified and the remaining 96 cells stay blocked;
- the matrix/support documentation is corrected because current evidence disproves an existing classification;
- one or more concrete implementation/security defects are found and separately registered as M127+ corrective plans.

M126 itself MUST NOT hide a production defect by folding an unrelated implementation patch into the audit.

## 2. Hard invariants

M126 preserves and re-verifies:

- Proposal 170 policy remains in `emissary-cli/src/i2pcontrol/**` wherever possible;
- no broad router-core refactor is authorized by this milestone;
- API version 1-only authentication behavior remains exact;
- HTTPS-only I2PControl serving and M107/M108 managed-TLS fail-closed behavior remain intact;
- protected methods cannot cross the control boundary without one unambiguous valid token;
- no fabricated RouterInfo, ClientServicesInfo or TunnelManager runtime state;
- no `accept_inert`, success-before-commit, or silent downgrade semantics;
- unsupported tunnel/options fail before avoidable runtime allocation or secret generation;
- AddressBook writes remain confined, serialized and durable according to their accepted owner;
- server destination/key material remains secret-safe and transactionally owned;
- M123 cancellation terminalization remains exact;
- no direct I2P-to-clearnet DNS fallback or expansion of accepted local-target boundaries;
- existing tunnel admission/cardinality/framing/lifetime controls remain effective;
- feature-disabled/default Emissary behavior is not contaminated by Proposal-specific production logic;
- historical closure records remain historical evidence and are not rewritten to conceal later findings.

M126 MUST NOT introduce a token-expiry requirement absent from the pinned base I2PControl contract. Security hardening must distinguish protocol requirements from optional local policy.

## 3. Work packages

### WP1 — Freeze and reconcile the normative surface

At the exact reviewed head:

1. Pin the May 20, 2026 Proposal 170 text and the current base I2PControl API-1 authentication/JSON-RPC contract in the closure evidence.
2. Reconstruct the Proposal 170 inventory independently of handler code:
   - RouterInfo additions;
   - AddressBook modes, books, subscriptions and SetConfig keys;
   - TunnelManager types, actions and option/type applicability cells;
   - ClientServicesInfo selectors.
3. Reconcile that inventory against `095-full-support-matrix.toml`, `105-residual-option-audit.toml`, `110-completion-ledger.toml`, M121/M125 corrections and current code.
4. Recompute the matrix counts mechanically. Either prove `284 / 96 / 460` or update the active matrix/evidence/docs atomically with an explicit reason for every changed cell.
5. Treat parser acceptance, deserialization, aliases, schema presence and fake adapters as insufficient for `apply`.

No applicability change may be made merely to reduce the blocked count or make a closure claim easier.

### WP2 — Trace every claimed production owner

For each `apply` family, record a source-to-runtime trace:

```text
JSON-RPC request
  -> authentication/authorization boundary
  -> method/domain validation
  -> production adapter/backend
  -> authoritative Emissary owner
  -> durable/live side effect or observation
  -> truthful RPC result/error
```

Required focus areas:

- `RouterInfo` request-time/live sources;
- `AddressBook` CRUD, subscriptions and SetConfig persistence/runtime effects;
- `TunnelManager` create/edit/get/start/stop/restart/delete and `All` behavior;
- `ClientServicesInfo` actual configured/enabled service state;
- startup composition in `emissary-cli/src/main.rs` only to the extent needed to prove the production adapters are actually installed.

Explicitly search for and reject production paths that:

- instantiate fake/test adapters;
- maintain an I2PControl-only shadow lifecycle that can diverge from the runtime owner;
- return configured state as running state;
- manufacture neutral/default values where the contract requires live observation;
- acknowledge mutation before authoritative commit;
- silently ignore accepted parameters.

Any such production defect is a stop condition requiring a focused M127+ corrective plan.

### WP3 — Authentication, TLS, JSON-RPC and resource adversarial qualification

Re-run and extend black-box HTTPS tests against the real I2PControl server composition.

At minimum prove:

- `Authenticate` accepts only the pinned API version and returns the expected token/result shape;
- every protected method rejects missing and invalid tokens;
- malformed token values fail deterministically;
- header/parameter token disagreement cannot choose a weaker credential or bypass authentication;
- alternate JSON-RPC request shapes, notifications and batches are either handled according to the supported contract or rejected without bypassing the protected-method boundary;
- JSON-RPC IDs are preserved where required and errors have the correct code/shape;
- internal Rust errors, filesystem paths, secrets, tokens and debug representations are not exposed in RPC responses/logs;
- the listener is TLS-only in production composition and managed-TLS failure does not fall back to plaintext;
- request-body, connection/concurrency, authentication-throttle, timeout and shutdown bounds remain effective under malformed/slow/repeated requests.

Do not add non-spec authentication semantics solely to make the test suite stricter.

### WP4 — AddressBook operational and security requalification

Exercise the production AddressBook owner through RPC and normal resolver integration.

Required positive evidence:

- Get/Put/Delete behavior for every supported book/mode;
- cross-book precedence remains consistent with the normal resolver;
- SetSubscriptions reaches the real subscription configuration owner;
- all 13 supported SetConfig keys reach their intended runtime/durable effect;
- committed changes survive reload/restart when persistence is part of the contract.

Required negative evidence:

- malformed hostnames/destinations are rejected before mutation;
- path/config values cannot escape accepted filesystem confinement;
- failed persistence does not report success or leave partially committed visible state;
- concurrent mutations are serialized or otherwise atomic according to the accepted store contract;
- symlink/type/permission failures are fail-closed and leave the prior good state intact;
- failed subscription/config changes do not create resolver/runtime divergence.

A second I2PControl-private address-book truth source is not acceptable if it can diverge from the normal resolver owner.

### WP5 — TunnelManager operational and security requalification

For every tunnel type/action currently classified `apply`, prove the RPC controls the real runtime backend rather than only a definition/inventory record.

Required lifecycle coverage:

- create;
- get;
- edit, including rename/collision behavior;
- start;
- stop;
- restart;
- delete;
- canonical `All` semantics only for actions where Proposal 170 permits them;
- startup-visible versus control-plane-created inventory coherence;
- persistence/reload behavior where the accepted implementation claims it.

Required adversarial coverage:

- duplicate names and edit-to-existing-name collisions;
- invalid/unsupported tunnel types and option combinations;
- all currently `blocked_primitive` options fail before avoidable runtime allocation, listener publication, destination-key generation or durable mutation;
- cancellation at start/edit/restart commit boundaries preserves M123 terminal-state guarantees;
- failed runtime start/edit rolls back definition/secret state exactly;
- concurrent lifecycle mutations preserve per-name exclusion and bounded resource ownership;
- server admission/cardinality/task limits remain bounded;
- local-target, HTTP framing/privacy, IRC lifetime/DCC/CTCP and Streamr fanout/subscriber controls retain the accepted security properties;
- secrets and private destinations do not enter generic RPC output, logs or unconfined persistence.

For each blocked option family, re-check whether a neutral authoritative owner now exists. If not, keep it blocked. If it does, do not implement it in M126; register a narrowly bounded successor plan with the exact owner and runtime semantics.

### WP6 — RouterInfo and ClientServicesInfo truthfulness

For every currently available RouterInfo selector:

- verify exact wire name/casing/type/presence semantics;
- identify the authoritative live owner;
- prove request-time values change when the underlying state changes where feasible;
- preserve any protocol-permitted neutral disposition exactly, without generalizing neutral constants to unavailable observations;
- verify mutating selectors such as log clearing have the claimed side effect and failure semantics.

For ClientServicesInfo:

- verify all six selectors use actual configured/enabled/runtime service state;
- disabled/unconfigured services are not reported as active;
- service metadata is never exposed through an unauthenticated path;
- startup/reload changes are reflected without stale I2PControl-only shadow state.

### WP7 — Containment and dependency re-audit

Compare the current Proposal 170 production delta against the accepted containment authority.

Classify every path outside `emissary-cli/src/i2pcontrol/**` as one of:

- neutral existing-owner capability seam;
- application composition wiring;
- accepted exact dependency boundary;
- tests/docs/planning.

Any Proposal-shaped policy, duplicate business logic or unexplained core/util dependency expansion outside that boundary blocks M126 closure and requires a separate corrective plan.

Re-check that the optional exact `yosemite-i2pcontrol` alias remains isolated and that ordinary Yosemite use is not globally patched/replaced.

### WP8 — Active documentation and evidence reconciliation

Reconcile current authority across:

- `plans/registry.md`;
- this roadmap;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`;
- `plans/implementation/i2pcontrol-proposal-170/105-residual-option-audit.toml`;
- `plans/implementation/i2pcontrol-proposal-170/110-completion-ledger.toml`;
- active repository guidance such as `AGENTS.md` where it states current Proposal 170 counts/status.

Known entry condition: `AGENTS.md` still advertises the older post-M113 `312/70/458` state while the post-M125 registry authority is `284/96/460`. M126 must remove that active inconsistency without rewriting historical records.

Active documentation MUST NOT say or imply “full Proposal 170 support” while any applicable cell remains `blocked_primitive`.

## 4. Verification commands

At minimum run and record exact outcomes for:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-core
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Add focused current-head tests for authentication/server, AddressBook, TunnelManager, RouterInfo and ClientServicesInfo only where the broad suites do not already prove the required invariant.

If an existing test name/target has changed, record that fact and use the current equivalent rather than weakening the gate. A failed broad gate remains a recorded failure even if a focused rerun passes.

External/reference-router traffic is not a prerequisite for claiming the **current implemented subset** remains internally operational unless the relevant capability specifically depends on external interoperability, but existing M038/M104/M114 interoperability evidence must not be cited as current-head proof if its production path materially changed afterward.

## 5. Evidence rubric

### Positive evidence required

M126 closure must contain:

1. exact reviewed head and dependency revisions;
2. pinned-spec inventory and recomputed matrix counts;
3. requirement-to-production-owner trace for every claimed family;
4. black-box HTTPS/auth/JSON-RPC results;
5. AddressBook mutation/persistence/restart evidence;
6. TunnelManager lifecycle, rollback, cancellation and unsupported-option evidence;
7. RouterInfo/ClientServicesInfo live-source evidence;
8. security/adversarial findings with severity;
9. exact changed-path containment classification;
10. documentation/evidence consistency check;
11. every verification command and result;
12. final disposition and any newly registered corrective plans.

### Evidence that invalidates closure

Any of the following prevents a clean requalification:

- a protected RPC succeeds without one valid unambiguous credential;
- a mutating RPC reports success without its authoritative side effect being committed;
- production state is supplied by a fake, inert fallback or diverging shadow model;
- a currently blocked option allocates/publishes/generates secrets before returning unsupported;
- a claimed RouterInfo/service value is fabricated rather than sourced according to its disposition;
- AddressBook durable/runtime state can diverge after a reported-success mutation;
- cancellation can strand a nonterminal tunnel generation or secret state;
- a high/medium Proposal-scoped security/anonymity issue remains unresolved;
- the active matrix and active documentation disagree;
- an unexplained Proposal-specific production change exists outside the accepted containment boundary.

## 6. Acceptance criteria

M126 closes successfully only when all are true:

1. the pinned Proposal 170 inventory is independently reconciled;
2. current matrix counts are mechanically reproduced or truthfully corrected with cell-level evidence;
3. every currently claimed `apply` surface reaches a real authoritative production owner and has current-head executable evidence appropriate to that surface;
4. no unauthenticated or ambiguous-credential protected-method path exists;
5. TLS, request-resource and error/secret boundaries remain fail-closed;
6. AddressBook operations are authoritative, confined, durable and failure-atomic;
7. supported TunnelManager actions/types are operational and lifecycle/rollback/cancellation semantics remain truthful;
8. blocked options remain fail-before-effect unless a separately planned primitive becomes available;
9. RouterInfo and ClientServicesInfo are wire-correct and source-truthful;
10. no open high/medium Proposal-scoped security defect remains hidden by the qualification claim;
11. every non-I2PControl production delta is explicitly justified by accepted containment authority;
12. active docs and planning evidence agree on current partial-support status and counts;
13. any concrete production defect discovered has a narrowly scoped M127+ plan registered before M126 is closed.

A successful M126 closure may state that the **implemented subset** is operationally/security qualified against the pinned May 20, 2026 proposal. It may not state full Proposal 170 compliance while blocked applicable cells remain.

## 7. Stop conditions and successor-plan rule

Stop the requalification and register a separate corrective plan when any current-head review proves:

- wire contract mismatch;
- authentication/TLS boundary defect;
- success-before-commit or shadow-state behavior;
- AddressBook persistence/confinement/atomicity defect;
- TunnelManager lifecycle/resource/security defect;
- RouterInfo/ClientServicesInfo fabrication or stale-owner defect;
- containment regression requiring production code changes outside trivial test/evidence plumbing;
- a previously blocked primitive has become genuinely implementable and requires production integration.

Successor plans MUST identify exact files, owner, invariant, tests and containment budget. Do not pre-register speculative M127+ work before the defect/capability is evidenced.

## 8. Explicit exclusions

M126 does not authorize:

- implementation of the 96 currently blocked cells;
- new tunnel types outside the pinned Proposal 170 inventory;
- broad router/core redesign to manufacture missing primitives;
- Java-I2P-specific behavior that Emissary cannot truthfully own;
- token expiry or other base-I2PControl policy not present in the pinned contract;
- branch-protection/CI administration;
- unrelated dependency upgrades, cleanup or formatting churn;
- upstream issue/PR/review/contact/submission/release/merge/adoption activity.

## 9. Commit shape

M126 should execute as a bounded evidence/corrective-planning sequence:

1. **qualification evidence/tests** — current-head inventory, focused regression tests and source-owner evidence; no opportunistic production feature implementation;
2. **corrective plan registration if required** — one or more M127+ plans for concrete findings, before production fixes begin;
3. **closure/docs reconciliation** — closure record, authoritative counts/status and active documentation updated only after the evidence gate is satisfied or the blocked disposition is recorded.

Historical closure files are immutable evidence except for explicit errata conventions already accepted by the repository.

## 10. Internal-only boundary

No upstream issue, pull request, review request, submission, merge/adoption request, branch/tag push, release, contribution package or maintainer contact is authorized. External specifications, source trees and reference routers are read-only evidence only.
