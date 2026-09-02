# I2PControl Proposal 170 Full-Support Completion Roadmap

Status: active; M095-M096 and M098-M109 and M115 closed, M097/M104 closed as blocked; **M110 ready**; M111-M114 remain roadmap-defined and blocked on primitive evidence

Planning origin: M094 closed planning head `630a8fd1cd4e5943fcde0b5c16f5fc1e88b5d207`.

Current corrective baseline: `ecb2245` — M115 implementation head; closure and planning reconciliation follow.

Pinned external authority:

- I2P Proposal 170, `I2PControl Expansion`;
- status: Open;
- pinned revision: `2026-05-20`.

Canonical/internal authority:

- `plans/000-long-term-specification.md`;
- `plans/001-terminology-and-domain-model.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- ADR-0001/0002/0003/0004;
- M061/M062/M063 containment authority;
- M093 current tunnel security reclosure;
- M095 machine-readable full-support matrix;
- M105 residual-option audit;
- M107-M109 closure evidence;
- M115 corrective plan for post-M109 lifecycle/composition defects.

All external specifications, source trees, issues, pull requests and reference routers are read-only evidence. All repository writes remain internal to `eggstack/emissary`. No upstream contribution/review/merge/contact activity is authorized.

## 1. Purpose

Move the internal fork from truthful partial Proposal 170 support to full support against the pinned revision while keeping Proposal 170 administrative/application policy concentrated under `emissary-cli/src/i2pcontrol/**` and preserving the already security-reviewed router/core boundary.

This roadmap is Proposal-170-only. It is not a general I2PControl parity program, not a router redesign, and not an upstream contribution program.

Full support means real wire/runtime semantics. Parser acceptance, raw persistence, fake/default state, fail-before-allocation rejection, or backend registration alone never counts as operational support.

## 2. Current production state

After M109 the fork has:

- RouterInfo: 43 canonical Proposal 170 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable;
- AddressBook CRUD, subscriptions, all 13 SetConfig keys, independent books and deterministic private→local→router→published effective lookup;
- all 12 canonical TunnelManager tunnel types with real data planes;
- all seven canonical TunnelManager action handlers;
- M097 common applied subset: `TunnelLength`, `TunnelQuantity`, typed `EncType`;
- M098 bounded client proxy/outproxy/auth/privacy subset;
- M099 bounded server presentation/access/filter/admission/rate subset;
- M106 six TCP-client `DelayOpen` cells;
- all six ClientServicesInfo selectors;
- API version 1-only authentication;
- bounded HTTPS serving, authentication/throttling, managed TLS and M107/M108 private-key hardening;
- fail-before-allocation behavior for unsupported residual options;
- startup-configured generic tunnel named lifecycle and mixed `All=true` semantics from M109.

Current M095 matrix:

- 70 option rows × 12 canonical tunnel types = 840 cells;
- 224 `apply`;
- 158 applicable `blocked_primitive`;
- 458 `not_applicable`;
- 0 `planned_apply`, unknown, unsupported, or accept-inert cells.

Post-M109 review found a separate corrective gate, now closed by M115:

- the M109 lifecycle-controlled startup path was selected whenever the feature was compiled rather than only when runtime I2PControl was enabled;
- lifecycle state could report synthetic `Starting` on internal mutex contention;
- the controlled startup-client shared Yosemite session could not recover cleanly from initial creation failure and lacked explicit final-member teardown ownership.

M115 closed all three findings with runtime-gated composition, atomic last-committed lifecycle snapshots, and a bounded retryable shared-session owner.

Remaining completion gates are therefore:

1. **Residual option semantics:** 158 applicable option/type cells remain owned by M110-M113;
2. **Final evidence:** public/reseeded/reference-router certification remains M114-only after the residual option gates close.

## 3. Ownership boundary

### Preferred ownership

Proposal 170 business/admin/application policy belongs under:

`emissary-cli/src/i2pcontrol/**`

This includes JSON-RPC parsing/validation, option capability policy, TunnelManager durable definitions, I2PControl-created runtime ownership, AddressBook authority, RouterInfo mapping/samplers, ClientServicesInfo aggregation, authentication/TLS, I2PControl-specific secret stores, and interoperability/support matrices.

### Neutral CLI-tunnel exception

M109 introduced the bounded neutral startup lifecycle seam in the existing CLI tunnel owner. M115 may correct only that same seam:

- `emissary-cli/src/main.rs` for runtime composition selection;
- `emissary-cli/src/tunnel/client.rs` for neutral startup client lifecycle/session ownership;
- `emissary-cli/src/tunnel/server.rs` for neutral startup server lifecycle state.

Proposal 170 policy remains under I2PControl. This exception does not authorize rewriting startup configuration, adding Proposal-shaped tunnel types/options to the CLI layer, or generalizing the startup session owner into M110's Proposal `Shared` implementation.

### Lower-layer exception rule

Any future production change outside I2PControl is allowed only when all are true:

1. required behavior belongs to an existing canonical owner;
2. no truthful I2PControl-local implementation exists;
3. exact paths/owners are named before implementation;
4. the seam is neutral/reusable rather than Proposal-170-shaped;
5. behavior is bounded and does not silently change unrelated router decisions;
6. M061/M062 containment is amended explicitly rather than implicitly;
7. a registered plan authorizes the exact change.

M102's neutral network-error observation and M109/M115's neutral startup lifecycle seam are the models. Neither creates a general license for core changes.

### Dependency rule

No milestone automatically authorizes vendoring/forking/patching Yosemite, path/git dependency overrides, a parallel SAM implementation, Proposal-170-shaped APIs in `emissary-core`, or dependencies added merely to make matrix counts green.

M111 remains blocked until the accepted Yosemite public API actually exposes the needed session-wire semantics or a separately accepted architecture decision explicitly changes this rule.

## 4. Cross-cutting invariants

All remaining work MUST preserve:

- exact pinned wire names/types/actions/presence semantics;
- API 1-only negotiation;
- no fabricated state or inert accepted configuration;
- every `apply` cell changes real runtime behavior;
- difficulty is not evidence of `not_applicable`;
- Java implementation machinery is not automatically a portable Proposal requirement;
- startup/control-plane/configuration owners remain explicit;
- feature-disabled and feature-compiled/runtime-disabled Emissary gain no I2PControl-only runtime behavior;
- direct I2P proxy traffic never falls through to clearnet DNS;
- clearnet proxy traffic requires explicit I2P outproxy routing;
- trusted Yosemite-derived remote peer identity;
- literal-loopback/local-target confinement from M090/M093 unless a separate accepted security decision changes it;
- bounded transactional server admission;
- HTTP/IRC/Streamr anonymity/resource/filter protections;
- secret and path-valued configuration redaction/confinement;
- restrictive managed I2PControl private-key handling from M107/M108;
- no silent LeaseSet security downgrade;
- lifecycle generation isolation and bounded cancellation;
- no lock across unrelated network I/O, sleeps, joins or cancellation waits;
- external interaction remains read-only/internal-only.

## 5. Explicit non-goals

This workstream MUST NOT implement unrelated base I2PControl methods, API 2, non-Proposal tunnel fields/types/actions, startup-config rewrites, router-global key/profile/plugin/session machinery for parity, unrelated protocol features, router-wide banning, relaxed AddressBook confinement, weakened local-target/proxy/LeaseSet security, frontend coupling, hosted CI/fuzz/release infrastructure, or upstream interaction.

## 6. Residual option partition

The authoritative cell-level evidence remains M095/M105. The post-M106 residual is 158 cells.

| Roadmap owner | Residual families | Current maximum cells | Primary blocker |
|---|---|---:|---|
| M110 | `Shared`; `NewDest`; `PersistentClientKey`; `PrivKeyFile` | 31 | bounded shared-session/client-secret/key ownership |
| M111 | `UseSSL`; `TunnelVariance`; `TunnelBackupQuantity`; `SigType`; `CustomOptions` | 44 | accepted Yosemite public session-wire capability |
| M112 | `UseOutproxyPlugin`; `SSLProxies`; `JumpList`; `ConnectDelay`; `Profile`; remaining `DelayOpen`; `Reduce*`; `Close*` | 62 | client runtime/proxy semantics and applicability |
| M113 | `AllowInternalSSL`; `UniqueLocalAddressPerClient`; `MultiHoming`; `EncryptLeaseSet`; `OptionalLookup`; `LeaseSetClientAuths` | 21 | server presentation/routing and LeaseSet security owners |

31 + 44 + 62 + 21 = 158 current blocked cells. M115 owns none of them.

A cell may become `not_applicable` only with affirmative pinned/reference evidence. A cell may remain blocked if a required safe primitive does not exist; in that case M114 cannot become ready and full support cannot be claimed.

## 7. Ordered milestone sequence

```text
M108 managed TLS corrective                         [CLOSED]
  |
  v
M109 startup-managed action semantics               [CLOSED]
  |
  v
M115 M109 runtime/lifecycle corrective              [CLOSED]
  |
  v
M110 shared session + destination/key ownership     [READY]
  |
  v
M111 SAM session-wire options                       [PROPOSED / BLOCKED ON DEPENDENCY]
  |
  v
M112 client proxy/session-lifecycle residuals       [PROPOSED / BLOCKED]
  |
  v
M113 server presentation + LeaseSet residuals       [PROPOSED / BLOCKED]
  |
  | zero applicable residual cells + no open corrective
  v
M114 live/reference interoperability + reclosure    [PROPOSED / BLOCKED]
```

M110-M114 were numbered before the M109 post-closure defects were discovered. Their identifiers remain stable; M115 is inserted into execution order without renumbering those plans.

Only M110 is registered in `plans/registry.md` as dependency-ready.

## 8. M109 — startup-managed tunnel action semantics

Plan: `plans/implementation/i2pcontrol-proposal-170/109-startup-managed-tunnel-action-semantics-corrective.md`

Status: **closed**; closure: `plans/closure/i2pcontrol-proposal-170/109-closure.md`.

M109 established neutral startup lifecycle control, truthful-intent inventory mapping, named canonical lifecycle, mixed `All=true`, and the immutable startup edit/delete ownership disposition without changing the option matrix.

Post-closure findings are not retroactively folded into M109; planning governance requires a new corrective pass, M115.

## 9. M115 — M109 runtime-disable and lifecycle-truthfulness corrective

Plan: `plans/implementation/i2pcontrol-proposal-170/115-m109-runtime-disable-and-lifecycle-truthfulness-corrective-pass.md`

Status: **closed**; closure: `plans/closure/i2pcontrol-proposal-170/115-closure.md`.

Objective:

- make runtime `i2pcontrol.enabled` authoritative for selecting the M109 lifecycle-controlled startup path;
- preserve the historical startup client/server manager path when runtime disabled;
- replace contention-based synthetic `Starting` with truthful lifecycle snapshots;
- make the controlled startup-client one-session owner retryable after creation failure;
- preserve one shared session while members are active and release it after the final member stops;
- preserve M109 named lifecycle/`All=true`, M093 security, and `224 / 158 / 458`.

Exact non-I2PControl production authority is limited to `emissary-cli/src/main.rs` and `emissary-cli/src/tunnel/{client,server}.rs`.

M115 does not implement Proposal option `Shared` and does not create a general I2PControl session registry.

## 10. M110 — shared session and destination/key ownership

Plan: `plans/implementation/i2pcontrol-proposal-170/110-shared-client-session-and-destination-key-ownership-completion.md`

Status: **ready / registered**.

Target: up to 31 residual cells.

M109 and M115 are closed. The M115 closure accepts the bounded I2PControl-local ownership model for independent M110 execution, the exact M095/M105 cell set remains frozen, and accepted Yosemite 0.7.0 exposes public `DestinationKind::Persistent` and `SessionOptions` primitives sufficient for the required destination-material handoff without dependency changes.

The neutral M115 startup-session owner is not evidence that Proposal `Shared` is implemented.

## 11. M111 — SAM session-wire option completion

Plan: `plans/implementation/i2pcontrol-proposal-170/111-sam-session-wire-option-completion.md`

Status: **proposed / dependency-blocked**.

Target: up to 44 residual cells. Completion requires real serialization through an accepted public Yosemite API. No raw-SAM construction, vendored/path Yosemite, or Proposal-shaped core seam is authorized.

## 12. M112 — client proxy/session-lifecycle residuals

Plan: `plans/implementation/i2pcontrol-proposal-170/112-client-proxy-and-session-lifecycle-residual-completion.md`

Status: **proposed / blocked**.

Target: up to 62 residual cells. Portable Proposal behavior must be separated from Java implementation mechanisms; proxy no-DNS/no-clearnet-fallback invariants remain mandatory.

## 13. M113 — server presentation/address-routing/LeaseSet residuals

Plan: `plans/implementation/i2pcontrol-proposal-170/113-server-presentation-address-routing-and-leaseset-residual-completion.md`

Status: **proposed / blocked**.

Target: up to 21 residual cells. LeaseSet encryption/client-auth requires a real accepted primitive with no downgrade; presentation/address-routing may not weaken M093 loopback/SSRF boundaries.

## 14. M114 — final live/reference reclosure

Plan: `plans/implementation/i2pcontrol-proposal-170/114-full-proposal-170-live-interoperability-and-final-reclosure.md`

Status: **proposed / blocked**.

Readiness requires:

- M115 and M110-M113 closed as applicable;
- M095 zero applicable blocked/planned/unsupported/unknown/inert cells;
- M105 zero unresolved applicable residuals;
- no open high/medium Proposal-170-scoped security/correctness corrective.

M114 performs no feature implementation. It re-verifies canonical inventory, full feature suite, runtime-disable isolation, mixed startup/control-plane lifecycle, all twelve data planes, reference interoperability, bounded public/reseeded truthfulness, security/anonymity, failure/recovery, and changed-path containment.

Only a successful M114 closure may state `full Proposal 170 support against pinned revision 2026-05-20`. That remains an internal pinned-revision claim, not upstream certification.

## 15. Security and anonymity requirements for remaining work

### M115 startup lifecycle corrective

- runtime-disabled composition must remain historical/legacy;
- cancellation before successor generation;
- no duplicate same-name runtime or shared session;
- no private server destination material in lifecycle handles;
- no fabricated state under contention;
- shared-session creation failure must be recoverable;
- final active controlled client stop must release the shared client session.

### Shared sessions/client keys

- deterministic compatibility key;
- bounded member/session count;
- member edit/restart cannot mutate another member's contract;
- owner-only secret material and no raw-config/log exposure;
- import paths confined and structurally validated.

### Session-wire options

- real dependency consumption before `apply`;
- no stringly raw-SAM bypass;
- no security downgrade;
- custom options cannot override typed security policy.

### Client lifecycle/proxy

- timers generation-local and bounded;
- no timer/task leaks;
- no DNS/clearnet fallback;
- no request-controlled plugin/module loading.

### Server/LeaseSet

- no request-selected LAN routing;
- trusted peer identity only;
- no encrypted/authenticated LeaseSet downgrade;
- bounded/redacted auth key state.

## 16. Persistence, failure, cancellation and contention

Remaining work must preserve the existing TunnelManager transaction model: validate before allocation/publication; failed edits preserve last-known-good state where possible; per-name lifecycle is serialized; workers/tasks are generation-local/cancellable; no lock spans unrelated network I/O/sleeps/joins/cancellation waits/filesystem sync; cleanup is bounded; unsupported options fail before allocation; no partial configuration is presented as success.

M115 additionally requires a retryable shared startup-client session owner with deterministic final-member release and no permanent poisoning after creation failure.

AddressBook/TLS owners are regression scope only unless a direct new defect is found.

## 17. Verification policy

Use focused tests plus existing feature-gated containment/matrix suites. Do not build new hosted orchestration infrastructure.

Baseline commands:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

M114 additionally runs `cargo test -p emissary-core` and records bounded disposable reference-router/public-network evidence.

The known stable/nightly rustfmt mismatch remains a tooling issue. Run and record it; do not retain unrelated formatter churn.

## 18. Planning/registration discipline

Per `plans/003-planning-process.md`:

- M109 remains historically closed;
- M115 is historically closed;
- M110 is the sole ready/registered successor handoff;
- M111-M114 remain roadmap/indexed but unregistered as active handoffs;
- M115 closure records the completed M110 readiness review;
- each later closure decides the next transition;
- blocked plans may not be executed simply because their files exist;
- material deviations require plan/ADR correction before code changes;
- closure records, not implementation assertions, decide completion.

## 19. Risks and deferred decisions

### M115 shared startup session ownership

The corrective must preserve one-session sharing for concurrently active startup clients without turning the neutral startup owner into M110's general Proposal `Shared` mechanism. If the accepted Yosemite `Session` cannot be dropped/recreated safely under the existing startup-client contract without new lower-layer work, M115 stops rather than broadening scope.

### Yosemite capability

M111 may remain blocked indefinitely if the accepted public dependency cannot express exact session-wire semantics. This is preferable to an unauthorized fork/vendor/parallel stack.

### Shared client identity

Cross-tunnel Proposal sharing creates timing/identity/lifecycle coupling. M110 must keep ownership bounded to I2PControl and treat compatibility conservatively.

### Startup edit/delete semantics

M109's closure retains immutable startup edit/delete ownership because the pinned contract does not define startup configuration mutation. A future pinned-contract change requires a separately numbered architecture/capability corrective.

### Java-specific option mechanisms

M112/M113 must avoid creating plugin/profile/multihoming/TLS frameworks unless the pinned contract requires portable behavior. Positive evidence is required for `not_applicable` reclassification.

### LeaseSet security

If no accepted primitive can supply encryption/client-auth semantics, M113 remains blocked and M114 cannot claim full support. No downgrade workaround is acceptable.

### Token lifetime / filesystem race residuals

Long-lived bearer tokens and the operator-controlled-base-directory path-race assumption remain separate hardening considerations absent new normative/architectural direction. They do not authorize scope expansion in M115 or M110-M114.

## 20. Final completion rule

The roadmap closes only with a successful M114 closure proving:

- exact pinned Proposal 170 inventory;
- M115 runtime-disable/lifecycle corrective closed;
- zero applicable residual option gaps;
- correct visible tunnel action semantics;
- production/live/reference interoperability;
- no high/medium Proposal-scoped security/correctness defects;
- minimal explained non-I2PControl production seams;
- feature-disabled and feature-compiled/runtime-disabled isolation;
- internal-only external interaction.

Until then, the official status remains **partial Proposal 170 support**.
