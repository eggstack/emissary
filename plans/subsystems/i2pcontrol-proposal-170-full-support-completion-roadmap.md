# I2PControl Proposal 170 Full-Support Completion Roadmap

Status: active; M095-M096 and M098-M109 closed, M097/M104 closed as blocked; M110-M114 roadmap-defined and blocked on predecessor/primitive evidence

Planning origin: M094 closed planning head `630a8fd1cd4e5943fcde0b5c16f5fc1e88b5d207`.

Current planning baseline: `2317705ef3bf21771715e243e87b62a6377a91eb` — post-M108 reconciliation.

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
- M097-M108 closure evidence.

All external specifications, source trees, issues, pull requests and reference routers are read-only evidence. All repository writes remain internal to `eggstack/emissary`. No upstream contribution/review/merge/contact activity is authorized.

## 1. Purpose

Move the internal fork from truthful partial Proposal 170 support to full support against the pinned revision while keeping Proposal 170 administrative/application policy concentrated under `emissary-cli/src/i2pcontrol/**` and preserving the already security-reviewed router/core boundary.

This roadmap is Proposal-170-only. It is not a general I2PControl parity program, not a router redesign, and not an upstream contribution program.

Full support means real wire/runtime semantics. Parser acceptance, raw persistence, fake/default state, fail-before-allocation rejection, or backend registration alone never counts as operational support.

## 2. Current production state

After M108 the fork has:

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
- fail-before-allocation behavior for unsupported residual options.

Current M095 matrix:

- 70 option rows × 12 canonical tunnel types = 840 cells;
- 224 `apply`;
- 158 applicable `blocked_primitive`;
- 458 `not_applicable`;
- 0 `planned_apply`, unknown, unsupported, or accept-inert cells.

Two independent completion gates remain:

1. **Action/inventory semantics:** startup-configured generic tunnels are visible to TunnelManager with named lifecycle and canonical `All=true` semantics. M109 closed this bounded corrective.
2. **Residual option semantics:** 158 applicable option/type cells still lack truthful runtime primitives. M110-M113 partition those residuals by actual ownership/security domain.

Full public/reseeded/reference-router certification remains open and belongs only to M114 after the first two gates are closed.

## 3. Ownership boundary

### Preferred ownership

Proposal 170 business/admin/application policy belongs under:

`emissary-cli/src/i2pcontrol/**`

This includes:

- JSON-RPC parsing/validation;
- option capability policy;
- TunnelManager durable definitions;
- I2PControl-created runtime ownership;
- AddressBook administrative authority;
- RouterInfo mapping/local samplers;
- ClientServicesInfo aggregation;
- authentication and managed I2PControl TLS;
- I2PControl-specific secret stores;
- interoperability fixtures and support matrices.

### Neutral CLI-tunnel exception

M109 may use the existing generic CLI tunnel layer as the canonical owner of startup tunnel lifecycle. The allowed design is a neutral reusable lifecycle handle in `emissary-cli/src/tunnel/**` plus composition wiring in `emissary-cli/src/main.rs`; Proposal 170 policy must remain in I2PControl.

This exception does not authorize rewriting startup configuration, adding Proposal-shaped tunnel types to the CLI layer, or making the CLI manager depend on I2PControl domain types.

### Lower-layer exception rule

Any future production change outside I2PControl is allowed only when all are true:

1. required behavior belongs to an existing canonical owner;
2. no truthful I2PControl-local implementation exists;
3. exact paths/owners are named before implementation;
4. the seam is neutral/reusable rather than Proposal-170-shaped;
5. behavior is bounded and does not silently change unrelated router decisions;
6. M061/M062 containment is amended explicitly rather than implicitly;
7. a registered plan authorizes the exact change.

M102's neutral network-error observation and M109's neutral startup lifecycle handle are the models. Neither creates a general license for core changes.

### Dependency rule

No milestone automatically authorizes:

- vendoring/forking/patching Yosemite;
- path/git dependency overrides;
- a parallel SAM implementation;
- Proposal-170-shaped APIs in `emissary-core`;
- dependencies added merely to make matrix counts green.

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
- feature-disabled/default Emissary gains no I2PControl-only runtime behavior;
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
- no lock across network I/O, sleeps, joins or cancellation waits;
- external interaction remains read-only/internal-only.

## 5. Explicit non-goals

This workstream MUST NOT:

- implement unrelated base methods such as `GetKeys`, `GetRate`, `RouterManager`, `NetworkSetting`, or `AdvancedSettings`;
- implement/advertise API 2;
- add non-Proposal-170 tunnel types/actions/statuses/fields;
- rewrite `router.toml` through I2PControl merely to manage startup tunnels;
- build a router-global destination/key/profile/plugin/multihoming subsystem solely for Proposal parity;
- add DCC/WEBIRC/SOCKS BIND/UDP ASSOCIATE absent a pinned requirement;
- add router-wide banning to support telemetry;
- relax AddressBook filesystem confinement merely to reproduce Java example paths;
- weaken local-target/proxy/LeaseSet security for reference parity;
- couple I2PControl to frontend state;
- add hosted CI/fuzz/coverage/release infrastructure for closure;
- initiate or prepare upstream review/merge/submission/contact.

## 6. Residual option partition

The authoritative cell-level evidence remains M095/M105. The post-M106 residual is 158 cells.

| Roadmap owner | Residual families | Current maximum cells | Primary blocker |
|---|---|---:|---|
| M110 | `Shared`; `NewDest`; `PersistentClientKey`; `PrivKeyFile` | 31 | bounded shared-session/client-secret/key ownership |
| M111 | `UseSSL`; `TunnelVariance`; `TunnelBackupQuantity`; `SigType`; `CustomOptions` | 44 | accepted Yosemite public session-wire capability |
| M112 | `UseOutproxyPlugin`; `SSLProxies`; `JumpList`; `ConnectDelay`; `Profile`; remaining `DelayOpen`; `Reduce*`; `Close*` | 62 | client runtime/proxy semantics and applicability |
| M113 | `AllowInternalSSL`; `UniqueLocalAddressPerClient`; `MultiHoming`; `EncryptLeaseSet`; `OptionalLookup`; `LeaseSetClientAuths` | 21 | server presentation/routing and LeaseSet security owners |

31 + 44 + 62 + 21 = 158 current blocked cells.

These are planning ownership partitions, not promises that every cell will become `apply`. A cell may become `not_applicable` only with affirmative pinned/reference evidence. A cell may remain blocked if a required safe primitive does not exist; in that case M114 cannot become ready and full support cannot be claimed.

## 7. Ordered milestone sequence

```text
M108 managed TLS corrective                         [CLOSED]
  |
  v
M109 startup-managed action semantics               [CLOSED]
  |
  v
M110 shared session + destination/key ownership     [PROPOSED / BLOCKED]
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
  | zero applicable residual cells + no open security corrective
  v
M114 live/reference interoperability + reclosure    [PROPOSED / BLOCKED]
```

Only the next dependency-ready plan is registered in `plans/registry.md`, per planning governance. M110-M114 are indexed here and in the implementation README but are not active handoffs; no successor is ready after M109 closure.

## 8. M109 — startup-managed tunnel action semantics

Plan:

`plans/implementation/i2pcontrol-proposal-170/109-startup-managed-tunnel-action-semantics-corrective.md`

Status: **closed**; closure: `plans/closure/i2pcontrol-proposal-170/109-closure.md`.

Objective:

- add the smallest neutral CLI-tunnel lifecycle handle over existing reusable startup runtime primitives;
- expose truthful runtime state to I2PControl;
- make named canonical start/stop/restart act on visible startup tunnels;
- make `All=true` include startup and control-plane-owned targets exactly once;
- preserve automatic startup behavior and no `router.toml` mutation;
- directly resolve the pinned-contract disposition of edit/delete for startup-origin visible definitions.

Expected non-I2PControl production paths are limited to `emissary-cli/src/tunnel/client.rs`, `emissary-cli/src/tunnel/server.rs`, and `emissary-cli/src/main.rs`. No core/util/dependency change is authorized.

M109 does not change M095 option counts. Its closure records that M110 remains blocked pending its independent ownership and accepted-Yosemite primitive gates.

Exit conditions are those in the plan. If correct edit/delete semantics require a competing durable overlay or `router.toml` rewrite, M109 stops and opens a separately numbered corrective instead of widening silently.

## 9. M110 — shared session and destination/key ownership

Plan:

`plans/implementation/i2pcontrol-proposal-170/110-shared-client-session-and-destination-key-ownership-completion.md`

Status: **proposed / blocked**.

Target: up to 31 residual cells.

Required design if unblocked:

- bounded I2PControl-local compatible shared-client session registry;
- bounded I2PControl-owned secret store for persistent/imported client destination material where required;
- generation-safe reference membership/teardown;
- confined `PrivKeyFile` import, copied into owned state rather than arbitrary external-file dependence;
- real `NewDest`/`PersistentClientKey` identity semantics;
- no router-global key subsystem.

Readiness requires M109 closure plus explicit acceptance of this ownership model and proof current accepted Yosemite APIs can consume the required destination material.

## 10. M111 — SAM session-wire option completion

Plan:

`plans/implementation/i2pcontrol-proposal-170/111-sam-session-wire-option-completion.md`

Status: **proposed / dependency-blocked**.

Target: up to 44 residual cells.

The only acceptable completion is real serialization into the Yosemite session creation path using an accepted public dependency API. No raw-SAM construction, vendored/path Yosemite, or Proposal-shaped core seam is authorized.

M111 becomes ready only when a released accepted Yosemite interface exposes the needed semantics or a separately accepted architecture decision explicitly changes dependency policy.

## 11. M112 — client proxy/session-lifecycle residuals

Plan:

`plans/implementation/i2pcontrol-proposal-170/112-client-proxy-and-session-lifecycle-residual-completion.md`

Status: **proposed / blocked**.

Target: up to 62 residual cells.

The plan must separate portable Proposal behavior from Java implementation mechanisms. It may implement bounded generation-local timers/policies in I2PControl where exact owners exist, and may evidence `not_applicable` only affirmatively. It must not build plugin, TLS MITM, profile, or global timer frameworks for parity.

Proxy work retains explicit I2P outproxy/no-DNS/no-clearnet-fallback invariants.

## 12. M113 — server presentation/address-routing/LeaseSet residuals

Plan:

`plans/implementation/i2pcontrol-proposal-170/113-server-presentation-address-routing-and-leaseset-residual-completion.md`

Status: **proposed / blocked**.

Target: up to 21 residual cells.

LeaseSet encryption/client-auth changes require a real accepted session/LeaseSet primitive and fail closed with no downgrade. Presentation/address-routing options may not weaken M093 loopback/SSRF/anonymity boundaries merely to reproduce Java local-interface machinery.

Any required core/dependency owner must be separately planned with exact paths before implementation.

## 13. M114 — final live/reference reclosure

Plan:

`plans/implementation/i2pcontrol-proposal-170/114-full-proposal-170-live-interoperability-and-final-reclosure.md`

Status: **proposed / blocked**.

Readiness requires:

- M109-M113 closed as applicable;
- M095 zero applicable blocked/planned/unsupported/unknown/inert cells;
- M105 zero unresolved applicable residuals;
- no open high/medium Proposal-170-scoped security corrective.

M114 performs no feature implementation. It re-verifies the canonical inventory, full feature suite, mixed startup/control-plane lifecycle, all twelve data planes, reference-router interoperability, bounded public/reseeded truthfulness, security/anonymity, failure/recovery, and changed-path containment.

Only a successful M114 closure may state `full Proposal 170 support against pinned revision 2026-05-20`. That remains an internal pinned-revision claim, not upstream certification.

## 14. Security and anonymity requirements for remaining work

### Startup lifecycle

- cancellation before successor generation;
- no duplicate same-name runtime;
- no private server destination material in lifecycle handles;
- default behavior unchanged when I2PControl is disabled.

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

## 15. Persistence, failure, cancellation and contention

Future implementation plans must preserve the existing TunnelManager transaction model:

- validate before allocation/publication;
- failed edits preserve last-known-good durable/runtime state where possible;
- per-name lifecycle serialized;
- timers/workers/tasks generation-local and cancellable;
- no lock across network I/O, sleeps, joins, cancellation waits or filesystem sync;
- bounded cleanup on stop/restart;
- unsupported options fail before allocation;
- no partial configuration presented as success.

AddressBook/TLS owners are regression scope only unless a direct new defect is found.

## 16. Verification policy

Use focused tests plus existing feature-gated containment/matrix suites. Do not build new hosted orchestration infrastructure.

Baseline commands:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
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

## 17. Planning/registration discipline

Per `plans/003-planning-process.md`:

- M109 is closed and no successor is registry-ready;
- M110-M114 remain roadmap/indexed but unregistered as active handoffs;
- M109 closure records whether M110 is truly dependency-ready;
- each later closure decides the next transition;
- blocked plans may not be executed simply because their files exist;
- material deviations require plan/ADR correction before code changes;
- closure records, not implementation assertions, decide completion.

## 18. Risks and deferred decisions

### Yosemite capability

M111 may remain blocked indefinitely if the accepted public dependency cannot express exact session-wire semantics. This is preferable to an unauthorized fork/vendor/parallel stack.

### Shared client identity

Cross-tunnel sharing creates timing/identity/lifecycle coupling. M110 must keep ownership bounded to I2PControl and treat compatibility conservatively.

### Startup edit/delete semantics

M109 directly rechecks whether immutable externally configured visible definitions are contract-valid. If durable mutation is required, a separate architecture decision/corrective is required; do not rewrite `router.toml` by implication.

### Java-specific option mechanisms

M112/M113 must avoid creating plugin/profile/multihoming/TLS frameworks unless the pinned contract requires portable behavior. Positive evidence is required for `not_applicable` reclassification.

### LeaseSet security

If no accepted primitive can supply encryption/client-auth semantics, M113 remains blocked and M114 cannot claim full support. No downgrade workaround is acceptable.

### Token lifetime / filesystem race residuals

Long-lived bearer tokens and the operator-controlled-base-directory path-race assumption remain separate hardening considerations absent new normative/architectural direction. They do not authorize scope expansion in M109-M114.

## 19. Final completion rule

The roadmap closes only with a successful M114 closure proving:

- exact pinned Proposal 170 inventory;
- zero applicable residual option gaps;
- correct visible tunnel action semantics;
- production/live/reference interoperability;
- no high/medium Proposal-scoped security defects;
- minimal explained non-I2PControl production seams;
- feature-disabled/default isolation;
- internal-only external interaction.

Until then, the official status remains **partial Proposal 170 support**.
