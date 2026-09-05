# I2PControl Proposal 170 Full-Support Completion Roadmap

Status: **active / partial; M139 ready / registered for post-lifecycle integrated requalification**

Current registered handoff:

- `plans/implementation/i2pcontrol-proposal-170/139-post-lifecycle-integrated-requalification-and-authority-rebase.md`.

Current machine authority:

- M095 matrix: `325 apply / 47 blocked_primitive / 468 not_applicable` across 840 TunnelManager option/family cells.

Current qualification lineage:

- M130 remains historical current-head runtime/security qualification authority until M139 closes;
- M131 remains residual applicability/primitive authority;
- M135/M136/M137/M134 are closed as complete and constitute the completed session-lifecycle implementation chain.

Pinned Proposal authority:

- I2P Proposal 170 revision `2026-05-20`, status Open.

All external specification/reference activity is read-only. Repository writes remain internal to `eggstack/emissary`; Yosemite writes require separate ADR-0005/Yosemite planning authority.

## 1. Purpose

Move the internal fork from truthful partial Proposal-170 support toward exact support while keeping Proposal-specific business/admin/application policy under `emissary-cli/src/i2pcontrol/**` wherever possible and preserving audited core/router boundaries.

Full support means real externally observable behavior. Parser acceptance, persistence, serializer reachability, dormant fields, fabricated defaults or approximate semantics do not count.

This roadmap is not a general base-I2PControl parity program, router redesign, frontend project or upstream contribution program.

## 2. Canonical/internal authority

Read in order:

1. `plans/000-long-term-specification.md`;
2. `plans/001-terminology-and-domain-model.md`;
3. `plans/002-long-term-roadmap.md`;
4. `plans/003-planning-process.md`;
5. ADR-0001 through ADR-0005;
6. M061/M062 containment;
7. M093 tunnel security;
8. M095 full-support matrix;
9. M105 residual audit;
10. M110 completion ledger;
11. M130/M131 historical/current authority chain;
12. M134/M135/M136/M137 lifecycle closures;
13. this roadmap, registry, and the registered implementation plan.

Historical closures remain immutable evidence. Later correctives/requalifications supersede only explicitly affected current claims.

## 3. Current support state

Current implemented/claimed subset includes:

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable;
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and deterministic precedence;
- all 12 canonical TunnelManager data planes and seven canonical actions for the claimed subset;
- all six ClientServicesInfo selectors;
- API-1 auth/version/token behavior required by the extension surface;
- finite token lifetime (M127), bounded JSON-RPC batches (M128), fail-closed non-loopback TLS (M129);
- shared-session/destination ownership from M110/M116;
- neutral variance/backup behavior from M118/M119;
- neutral live tunnel quantity + LeaseSet desired-count control from M135;
- Proposal `Reduce`/`ReduceCount`/`ReduceTime` for all seven client families from M136;
- Proposal `Close`/`CloseTime` for all seven client families plus authoritative idle termination cause from M137;
- Proposal `NewDest` for the six non-Streamr TCP client families from M134.

Current matrix:

- `325 apply`;
- `47 blocked_primitive`;
- `468 not_applicable`.

Full Proposal 170 support is **not** claimed.

M130 predates all post-M131 lifecycle production work and therefore remains historical qualification lineage rather than sufficient current-head qualification. M139 exists specifically to establish a new integrated current-head authority.

## 4. Ownership and containment

Proposal policy belongs under `emissary-cli/src/i2pcontrol/**`.

A production change outside that boundary is permitted only when:

1. behavior belongs to an existing canonical lower-layer owner;
2. no truthful I2PControl-local implementation exists;
3. exact paths are named before implementation;
4. the seam is neutral, not Proposal-shaped;
5. unrelated router behavior is unchanged;
6. M061/M062 exact-path evidence is amended;
7. a registered plan authorizes the change.

Accepted lifecycle exceptions are bounded:

- M135: neutral tunnel-pool/destination/LeaseSet desired-target primitive;
- M136: neutral SAM application-activity/idle-reduction owner;
- M137: neutral close-on-idle/reasoned-termination extension;
- M134: NewDest policy remains I2PControl-owned, with only the minimal application composition seam needed to share the volatile tracker with the neutral SAM observation source.

M139 authorizes no production-source or dependency change.

Yosemite remains the sole accepted SAM client for I2PControl. Exact Y005 stays isolated behind optional `yosemite-i2pcontrol`. No global patch, vendoring, path override, floating fork or parallel raw SAM implementation is authorized.

## 5. Cross-cutting invariants

All remaining work preserves:

- exact pinned names/types/actions/presence semantics;
- no fabricated/accept-inert support;
- every `apply` cell changes real runtime behavior;
- unsupported supplied values fail before allocation/effect;
- no direct-I2P-to-clearnet DNS/network fallback;
- trusted peer identity and Streamr producer isolation;
- local-target confinement unless a separately accepted neutral primitive proves equivalent safety;
- bounded admission/tasks/timers/state and generation-local cancellation;
- transactional edit/start/restart and last-known-good preservation;
- no lock across unrelated network/filesystem I/O, sleeps, joins or timer waits;
- secret/key/path redaction and confinement;
- no LeaseSet security downgrade or fabricated leases;
- feature-disabled/runtime-disabled isolation;
- no unrelated base method parity;
- no frontend coupling;
- external interaction read-only/internal-only.

## 6. Completed lifecycle line

The first combined attempts correctly stopped rather than approximate:

- M132 combined idle reduction/live target attempt — closed as blocked;
- M133 combined idle-close attempt — closed as blocked.

Direct reference research then enabled the corrected decomposition:

```text
M131 residual primitive re-freeze                     [CLOSED AS BLOCKED — 284/88/468]
  |
  +--> M132 combined reduction attempt                [CLOSED AS BLOCKED]
  +--> M133 combined close attempt                    [CLOSED AS BLOCKED]
  |
  v
M135 neutral live quantity + LeaseSet desired count   [CLOSED AS COMPLETE — 284/88/468]
  |
  v
M136 SAM activity + Reduce*                           [CLOSED AS COMPLETE — 305/67/468]
  |
  v
M137 Close* + reasoned termination                    [CLOSED AS COMPLETE — 319/53/468]
  |
  v
M134 NewDest on proven idle resume                    [CLOSED AS COMPLETE — 325/47/468]
  |
  v
M139 integrated current-head requalification          [READY / REGISTERED — ZERO PROMOTION]
```

The session-lifecycle focused roadmap is closed as complete. No lifecycle successor remains.

Historical planning mentioned a possible NewDest-corrective “M138”; M134 proved it unnecessary. No M138 implementation plan was registered. The unrelated post-lifecycle requalification therefore uses M139.

## 7. M139 — active requalification milestone

Plan:

- `139-post-lifecycle-integrated-requalification-and-authority-rebase.md`.

M139 is an invariant/qualification/roadmap-correction milestone with **zero support-promotion budget**.

It must:

- mechanically rederive `325/47/468` and the 47 residual cells;
- classify and correct stale current-head assertions in M126/M130-era tests while preserving historical milestone facts;
- re-prove M127 token, M128 batch and M129 TLS security behavior on the lifecycle head;
- re-prove M135→M136→M137→M134 lifecycle composition, including deterministic end-to-end idle-reduction → idle-close → `IdlePolicy` → resume/NewDest behavior and manual/restart negative behavior;
- reconcile M060/M061/M062 current containment/dependency guards without broadening allowances for convenience;
- run current production/live/adversarial/persistence/client-services/router-info suites;
- reconcile active docs/roadmaps/registry so one current authority is named;
- become the new current runtime/security qualification authority only if no high/medium defect remains.

M139 authorizes no production Rust, dependency, Yosemite, router, NetDb, crypto, transport, frontend or residual-capability work. A defect requiring production code causes M139 to stop and produce a separate corrective plan.

M139 must not register the next residual capability plan at closure.

## 8. Remaining 47 residual cells

All remain **unregistered** under M131 while M139 executes.

Expected current grouping, to be mechanically revalidated by M139:

| Primitive cluster | Blocked cells |
|---|---:|
| `SigType` destination signing | 10 |
| encrypted/authenticated LeaseSet cluster | 15 |
| streaming `Profile` | 7 |
| presentation `UseSSL` | 4 |
| `UseOutproxyPlugin` | 4 |
| HTTP `SSLProxies` + `JumpList` | 2 |
| `UniqueLocalAddressPerClient` | 2 |
| `MultiHoming` / `shouldBundleReplyInfo` | 2 |
| Streamr `ConnectDelay` | 1 |
| **Total** | **47** |

M095 machine authority wins over this prose if an exact row audit reveals a discrepancy; M139 must explain and correct the prose rather than force the matrix to match a summary.

## 9. Residual architecture expectations

The roadmap intentionally does not pre-register these clusters, but their ownership constraints remain:

- `SigType`: genuine destination/private-signing-key generation/signing support; no fake certificate-only support;
- encrypted/authenticated LeaseSets: genuine encrypted LS2/blinding/client-auth primitives required; no storage-only claim;
- `Profile`: neutral streaming behavior/config owner, not Proposal-shaped core API;
- `UseSSL`: application/local presentation TLS, not Yosemite SAM-control TLS;
- outproxy/plugin/HTTP proxy fields: must preserve I2P-only egress and avoid direct-clearnet fallback;
- `UniqueLocalAddressPerClient`: source-address behavior must preserve loopback/local-target confinement and portability;
- `MultiHoming`: reference `shouldBundleReplyInfo` LeaseSet reply behavior, not host-interface routing;
- Streamr `ConnectDelay`: remain blocked until exact datagram/session semantics are proven.

Each future cluster requires its own dependency-ready plan and exact path budget after M139 closure.

## 10. Historical authority

| Milestone | Current role |
|---|---|
| M061/M062 | containment/dependency authority |
| M093 | tunnel application/security boundary |
| M095 | machine-readable support matrix |
| M105 | residual-option audit |
| M110/M116 | shared-session/destination ownership |
| M117 | exact optional Yosemite dependency seam |
| M118/M119 | neutral variance/backup behavior |
| M121 | SigType and historical Close/NewDest truthfulness correction |
| M123 | cancellation/commit atomicity |
| M127 | finite token lifetime |
| M128 | bounded JSON-RPC batch conformance |
| M129 | fail-closed non-loopback TLS |
| M130 | historical integrated runtime/security qualification before M131/lifecycle production work |
| M131 | residual applicability/primitive authority |
| M135 | neutral live quantity/LeaseSet primitive |
| M136 | Reduce* implementation authority |
| M137 | Close*/termination-reason implementation authority |
| M134 | NewDest proven-resume implementation authority |
| M139 | active post-lifecycle integrated requalification handoff |

## 11. Successor readiness requirements

After M139 closes, no future residual primitive is dependency-ready until it defines:

- exact externally observable effect;
- canonical owner and exact path budget;
- validation-before-effect boundary;
- allocation/publication point;
- cancellation/generation/restart owner;
- bounded state/queue/timer semantics;
- lock/contention behavior;
- rollback/last-known-good behavior;
- security/secret implications;
- deterministic focused tests;
- reference/live interoperability requirements where applicable.

Serializer acceptance alone is not capability readiness.

## 12. Verification policy

M139 defines the immediate integrated verification baseline. Future implementation milestones continue to require:

- affected core/CLI checks and tests;
- feature-disabled checks;
- M061/M062 containment;
- M095/M105 matrix/residual guards;
- live/adversarial evidence where applicable;
- clippy and `git diff --check`;
- `cargo fmt --all -- --check` attempted with pre-existing stable/nightly drift recorded rather than normalized through unrelated churn.

No new hosted CI/fuzz/release orchestration is required by this roadmap.

## 13. Registration discipline

Per `plans/003-planning-process.md`:

- M139 is the only active Proposal-170 handoff;
- all 47 residual capability clusters remain unregistered while M139 is open;
- M139 closure must leave them unregistered even on success;
- selection of the next residual cluster is a separate post-requalification planning decision;
- material deviations require plan amendment before production code;
- closure evidence decides qualification and support truthfulness.

## 14. Final completion rule

Full Proposal 170 completion requires:

- zero applicable residual primitive gaps against the pinned revision;
- every applied cell backed by real runtime behavior;
- no high/medium Proposal-scoped correctness/security defect;
- bounded reference/live interoperability evidence;
- minimal explained non-I2PControl production seams;
- final whole-surface requalification after the last capability closes.

Until then the official status remains **partial Proposal 170 support**.