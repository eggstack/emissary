# I2PControl Proposal 170 Full-Support Completion Roadmap

Status: **active / partial; M140 closed as complete; M141 closed; M142 closed as complete (329/36/475); M143 closed as complete (330/35/475); no registered successor**

Current runtime/security qualification authority:

- M139 closure: `plans/closure/i2pcontrol-proposal-170/139-closure.md`.

Current residual applicability authority:

- M140 closure: `plans/closure/i2pcontrol-proposal-170/140-closure.md` (zero promotions; `325/40/475`).

Current registered handoff:

- none (M143 closed; M144-M152 deferred). Last closed: M143 `plans/implementation/i2pcontrol-proposal-170/143-streaming-profile-runtime-completion.md`.

Focused residual execution roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`.

Current machine authority after M143 closure:

- M095 matrix: `330 apply / 35 blocked_primitive / 475 not_applicable` across 840 TunnelManager option/family cells.

Pinned Proposal authority:

- I2P Proposal 170 revision `2026-05-20`, status Open.

All external specification/reference activity is read-only. Repository writes remain internal to `eggstack/emissary`; Yosemite writes require separate ADR-0005/Yosemite planning authority.

## 1. Purpose

Move the internal fork from truthful partial Proposal-170 support toward exact support while keeping Proposal-specific business/admin/application policy under `emissary-cli/src/i2pcontrol/**` wherever possible and preserving reviewed core/router boundaries.

Full support means real externally observable behavior. Parser acceptance, persistence, serializer reachability, dormant fields, fabricated defaults or approximate semantics do not count.

This top-level roadmap defines the completion standard and accepted authority chain. The detailed dependency/path/security plan for the remaining 47 blockers is now delegated to `i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`.

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
11. M121 truthfulness correction;
12. M130/M131 historical qualification/residual authority;
13. M135/M136/M137/M134 lifecycle closures;
14. M139 current integrated qualification closure;
15. residual primitive completion roadmap;
16. `plans/registry.md` and the single registered plan.

Historical closures remain immutable evidence. Later closures supersede only explicitly affected current claims.

## 3. Current support state

Current implemented/qualified subset includes:

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable;
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and deterministic precedence;
- all 12 canonical TunnelManager data planes and seven canonical actions for the claimed subset;
- all six ClientServicesInfo selectors;
- finite token lifetime (M127), bounded JSON-RPC batches (M128), fail-closed non-loopback management TLS (M129);
- shared-session/destination ownership from M110/M116;
- neutral variance/backup behavior from M118/M119;
- neutral live tunnel quantity + LeaseSet desired-count control from M135;
- Proposal `Reduce`/`ReduceCount`/`ReduceTime` for all seven client families from M136;
- Proposal `Close`/`CloseTime` for all seven client families plus authoritative idle termination cause from M137;
- Proposal `NewDest` for the six non-Streamr TCP client families from M134.

Current matrix after M140 closure:

- `325 apply`;
- `40 blocked_primitive`;
- `475 not_applicable`.

M140 made seven evidence-backed `blocked_primitive -> not_applicable` reclassifications with zero `apply` promotions (`325/40/475` is now current authority).

Full Proposal 170 support is **not** claimed.

## 4. Ownership and containment

Proposal policy belongs under `emissary-cli/src/i2pcontrol/**` wherever truthful implementation permits.

A production change outside that boundary is permitted only when:

1. behavior belongs to an existing canonical lower-layer owner;
2. no truthful I2PControl-local implementation exists;
3. exact files are named before implementation;
4. the seam is neutral, not Proposal-shaped;
5. unrelated router behavior is unchanged;
6. M061/M062 exact-path evidence is amended;
7. a registered plan explicitly authorizes the change.

M140 authorized no production code and made none. The deferred residual plans deliberately distinguish:

- I2PControl-local targets: M141, M142, M144, preferred M146;
- conditional small neutral core seams: M143, M145;
- security-sensitive exact-file infrastructure gates: M147, M149, M150, M151.

The existence of a deferred plan or its listing in M062 is planning-only bookkeeping. It does not authorize candidate production paths.

Yosemite remains the sole accepted SAM client for I2PControl. Exact Y005 stays isolated behind optional `yosemite-i2pcontrol`. No global patch, vendoring, path override, floating fork or parallel raw SAM implementation is authorized.

## 5. Cross-cutting invariants

All remaining work preserves:

- exact pinned names/types/actions/presence semantics;
- no fabricated/accept-inert support;
- every `apply` cell changes real runtime behavior;
- every `not_applicable` cell has affirmative Proposal/reference family evidence;
- unsupported supplied values fail before allocation/effect;
- no direct-I2P-to-clearnet DNS/network fallback;
- trusted peer identity and Streamr producer isolation;
- local-target confinement unless an explicitly accepted neutral primitive proves equivalent safety;
- bounded admission/tasks/timers/state and generation-local cancellation;
- transactional edit/start/restart and last-known-good preservation;
- no lock across unrelated network/filesystem I/O, sleeps, joins or timer waits;
- secret/key/path redaction and confinement;
- no LeaseSet security downgrade or fabricated leases;
- feature-disabled/runtime-disabled isolation;
- no unrelated base method parity;
- no frontend coupling;
- external interaction read-only/internal-only.

## 6. Completed lifecycle / qualification line

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
M134 NewDest on proven idle resume                    [CLOSED AS COMPLETE — 325/47/468 at M134 head]
  |
  v
M139 integrated current-head requalification          [CLOSED AS COMPLETE — ZERO PROMOTION]
  |
  v
M140 streaming applicability re-freeze               [CLOSED AS COMPLETE — 325/40/475, ZERO PROMOTION]
  |
  v
M141 HTTP unique local source address                [CLOSED AS COMPLETE — 327/38/475, 2 PROMOTIONS]
  |
  v
M142 HTTP SSLProxies + JumpList                      [CLOSED AS COMPLETE — 329/36/475, 2 PROMOTIONS]
  |
  v
M143 retained streaming Profile                     [CLOSED AS COMPLETE — 330/35/475, 1 PROMOTION]
```

M139 supersedes M130 for current-head runtime/security qualification only. M130 remains historical evidence. M131 remains historical residual authority except where M140 explicitly reclassifies seven cells to current-head `not_applicable` and where M141 promotes two cells to `apply`.

Historical planning mentioned a possible NewDest-corrective “M138”; M134 proved it unnecessary and no M138 implementation plan was registered.

## 7. Current residual completion line

The residual roadmap registers only the next dependency-ready plan:

```text
M139 current integrated qualification                  [CLOSED]
  |
  v
M140 streaming applicability re-freeze                 [CLOSED AS COMPLETE — 325/40/475, ZERO PROMOTION]
  |
  v
M141 UniqueLocalAddressPerClient                       [CLOSED AS COMPLETE — 327/38/475, 2 PROMOTIONS]
  |
  v
M142 HTTP SSLProxies + JumpList                       [CLOSED AS COMPLETE — 329/36/475, 2 PROMOTIONS]
  |
  v
M143 retained streaming Profile                       [CLOSED AS COMPLETE — 330/35/475, 1 PROMOTION]
  -> M144 application/presentation UseSSL              [DEFERRED]
  -> M145 reply LeaseSet bundling / MultiHoming        [DEFERRED]
  -> M146 real outproxy provider / UseOutproxyPlugin   [DEFERRED]
  -> M147 neutral destination signature suites         [DEFERRED / ZERO PROMOTION]
  -> M148 Proposal SigType                             [DEFERRED]
  -> M149 encrypted LeaseSet / EncryptLeaseSet         [DEFERRED]
  -> M150 OptionalLookup                               [DEFERRED]
  -> M151 LeaseSetClientAuths                          [DEFERRED]
  -> M152 final whole-surface requalification          [DEFERRED / ZERO PROMOTION]
```

M142-M152 are committed handoff documents but are not executable authority until the registry promotes them after their hard dependencies close (M143 closed; M144-M152 remain deferred). M145/M147/M149-M151 require explicit pre-registration amendments to freeze exact cells/files/spec/security decisions.

## 8. Remaining 38 residual cells after M141 closure

| Primitive cluster | Blocked cells |
|---|---:|
| `SigType` destination signing | 10 |
| encrypted/authenticated LeaseSet cluster | 15 |
| streaming `Profile` (retained `client` only) | 1 |
| presentation `UseSSL` | 4 |
| `UseOutproxyPlugin` | 4 |
| HTTP `SSLProxies` + `JumpList` | 2 |
| `MultiHoming` / `shouldBundleReplyInfo` | 2 |
| **Total** | **38** |

M095 machine authority wins over prose. M140 re-froze the seven Profile rows plus Streamr ConnectDelay using actual family constructor/runtime evidence and moved only affirmatively proven blockers to N/A with zero apply promotions. M141 promotes the two `UniqueLocalAddressPerClient` HTTP server cells with exact source-bind evidence.

## 9. Residual architecture selection

The detailed residual roadmap freezes the selected order and rationale:

- first correct applicability where actual Java family constructors/UDP ownership make a generic setter misleading;
- then exhaust I2PControl-local application/data-plane work (`UniqueLocalAddressPerClient`, HTTP residuals, presentation TLS, safe outproxy provider);
- introduce only the smallest neutral streaming/reply-bundling seams where exact runtime effects cannot live locally;
- stage destination signing infrastructure separately from Proposal SigType mapping;
- stage LeaseSet security confidentiality -> lookup -> client authorization;
- finish with a zero-production final current-head requalification.

This ordering minimizes contamination of upstream-reviewed Emissary code and prevents broad crypto/NetDB work from being used to claim several Proposal fields at once without separate runtime evidence.

## 10. Historical/current authority table

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
| M130 | historical integrated qualification before M131/lifecycle work |
| M131 | historical residual applicability/primitive authority |
| M135 | neutral live quantity/LeaseSet primitive |
| M136 | Reduce* implementation authority |
| M137 | Close*/termination-reason implementation authority |
| M134 | NewDest proven-resume implementation authority |
| M139 | current post-lifecycle runtime/security qualification authority |
| M140 | current residual streaming applicability authority (closed as complete; `325/40/475`; retained `Profile:client` frozen for M143) |
| M141 | sole registered residual capability handoff |

## 11. Successor readiness requirements

No deferred residual plan becomes executable until it defines, for its then-current codebase:

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

Security-sensitive core work additionally requires exact algorithm/spec/dependency decisions and exact-file M061/M062 authorization before registration. Serializer acceptance alone is not capability readiness.

## 12. Verification policy

Every future implementation milestone continues to require:

- affected core/CLI checks and tests;
- feature-disabled checks;
- M061/M062 containment;
- M095/M105 machine/residual guards;
- live/adversarial/reference evidence where applicable;
- clippy and `git diff --check`;
- `cargo fmt --all -- --check` attempted with pre-existing stable/nightly drift recorded rather than normalized through unrelated churn.

No new hosted CI/fuzz/release orchestration is required by this roadmap.

## 13. Registration discipline

Per `plans/003-planning-process.md`:

- M139 remains current runtime/security qualification authority; M140 is closed as residual streaming applicability authority;
- M141 is the only registered residual plan;
- M142-M152 are deferred/unregistered;
- after each closure, register only the next hard-dependency-ready plan;
- amend deferred path/security assumptions before registration where required;
- material deviations require plan amendment before production code;
- closure evidence decides qualification and support truthfulness.

## 14. Final completion rule

Full Proposal 170 completion requires M152 to prove:

- zero applicable residual primitive gaps against the pinned revision;
- every applied cell backed by real runtime behavior;
- every N/A cell backed by affirmative family/reference evidence;
- no high/medium Proposal-scoped correctness/security defect;
- bounded reference/live interoperability evidence;
- minimal explained exact-file non-I2PControl production seams;
- final M061/M062 containment/dependency isolation;
- no production changes hidden inside the qualification milestone.

Until M152 closes complete, the official status remains **partial Proposal 170 support**.