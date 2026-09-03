# I2PControl Proposal 170 Full-Support Completion Roadmap

Status: active; M095-M114, M115-M124 closed; M097/M104/M112/M113/M114 closed as blocked; focused M113 capability/crypto-ownership audit authorized but no successor implementation plan registered

Planning origin: M094 closed planning head `630a8fd1cd4e5943fcde0b5c16f5fc1e88b5d207`.

Current corrective baseline: `09247ccf8367a7b3a7050e0584614c4e59cafe8e` — post-M110 closure/containment head.
M116 implementation head: `626d76311a6dc142ecc07827845081b9a9f4c860`.
M117 implementation head: `22c893a`.
M118 implementation head: `e7f3e04`.
M112 implementation head: `5b2f3caa6af8767ef393254f20ca010211a8de3a`.
M113 implementation head: `82368ea` (closure-only; no production delta).

Pinned external authority:

- I2P Proposal 170, `I2PControl Expansion`;
- status: Open;
- pinned revision: `2026-05-20`.

Canonical/internal authority:

- `plans/000-long-term-specification.md`;
- `plans/001-terminology-and-domain-model.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- ADR-0001/0002/0003/0004/0005;
- M061/M062/M063 containment authority;
- M093 tunnel security reclosure;
- M095 machine-readable full-support matrix;
- M105 residual-option audit;
- M110 completion ledger;
- M109/M115/M110 historical closure evidence;
- M116 post-M110 corrective plan.
- M117 internal Yosemite fork pin and I2PControl adapter integration plan/closure.
- M118 neutral SAM tunnel-pool variance/backup capability plan/closure.

All external specifications, source trees, documentation, issues, pull requests and reference routers are read-only evidence. All repository writes remain internal to `eggstack/emissary`. No upstream contribution/review/merge/contact activity is authorized.

## 1. Purpose

Move the internal fork from truthful partial Proposal 170 support to full support against the pinned revision while concentrating Proposal 170 administrative/application policy under `emissary-cli/src/i2pcontrol/**` and preserving the already security-reviewed router/core boundary.

This roadmap is Proposal-170-only. It is not a general I2PControl parity program, router redesign, dependency-fork program, or upstream contribution program.

Full support means exact wire/runtime semantics. Parser acceptance, raw persistence, approximate behavior, fake/default state, or an unconsumed option never counts as operational support.

## 2. Current production state

The fork currently has:

- RouterInfo: 43 canonical Proposal 170 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable;
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and deterministic effective lookup;
- all 12 canonical TunnelManager tunnel types with real data planes;
- all seven canonical TunnelManager actions;
- M097/M098/M099/M106 bounded applied option subsets;
- all six ClientServicesInfo selectors;
- API version 1-only authentication;
- bounded HTTPS/authentication/throttling and M107/M108 managed-TLS hardening;
- M109 named startup lifecycle/mixed `All=true` and M115 runtime-disable/lifecycle corrections;
- M110 I2PControl-local shared-client session and destination/key ownership.

The current authoritative M095 matrix records:

- 70 option rows × 12 canonical tunnel types = 840 cells;
- 284 `apply`;
- 98 `blocked_primitive`;
- 458 `not_applicable`;
- 0 planned/unknown/unsupported/accept-inert cells.

M116 closed with exact counts of 248 `apply`, 134 `blocked_primitive`, and 458 `not_applicable`. M111 then applied 40 SAM session-wire cells, and M112 applied 24 portable TCP client lifecycle cells, leaving 312 `apply`, 70 `blocked_primitive`, and 458 `not_applicable`. Post-M110 review found and M116 corrected defects in:

- shared-session waiter lost wakeup;
- creator-cancellation reservation poisoning;
- collision-unsafe persistent identity fingerprinting;
- Streamr shared-session cross-producer application delivery;
- unproven `NewDest` lifecycle semantics;
- internal raw secret `Debug` derivation.

M116 was the corrective gate. Matrix correctness took precedence over preserving the post-M110 count; seven client `NewDest` cells moved to M112.

Remaining completion gates after M116 are:

1. M111 accepted Yosemite session-wire capability and M118 neutral tunnel-pool semantics;
2. M112 client proxy/lifecycle semantics (including the seven `NewDest` cells transferred by M116);
3. M113 server presentation/routing/LeaseSet security semantics;
4. M114 final live/reference/public/security/containment reclosure.

## 3. Ownership boundary

### Preferred ownership

Proposal 170 business/admin/application policy belongs under `emissary-cli/src/i2pcontrol/**`.

This includes JSON-RPC validation, option capability policy, TunnelManager durable definitions/runtime ownership, AddressBook administration, RouterInfo mapping/samplers, ClientServicesInfo aggregation, I2PControl TLS/authentication, I2PControl-specific secret stores, session registries and support/interoperability evidence.

### Historical neutral CLI-tunnel exception

M109/M115 introduced and corrected the bounded neutral startup lifecycle seam in:

- `emissary-cli/src/main.rs`;
- `emissary-cli/src/tunnel/client.rs`;
- `emissary-cli/src/tunnel/server.rs`.

That seam remains closed regression authority. M116 authorizes no changes there and must not generalize Proposal `Shared` into the startup owner.

### M116 ownership

M116 is intentionally I2PControl-only. Its canonical owners already exist in:

- `backends/runtime/session.rs`;
- `backends/runtime/client_listener.rs`;
- `backends/streamr.rs`;
- `client_secret_store.rs`;
- conditionally `backends/options.rs` and `production.rs` for the final `NewDest` transaction/disposition.

No lower-layer exception is needed.

### Lower-layer exception rule

Any future production change outside I2PControl requires all of:

1. behavior belongs to an existing canonical lower-layer owner;
2. no truthful local implementation exists;
3. exact paths are named before implementation;
4. the seam is neutral, not Proposal-shaped;
5. unrelated router behavior is unchanged;
6. M061/M062 is amended explicitly;
7. a registered plan authorizes the exact change.

M116 satisfies none of the reasons to invoke this exception, so no external path is authorized.

M118 is the separately registered neutral exception for generic SAM tunnel-pool variance
and standby/failover behavior. Its exact production paths and mechanical config-construction
seam are recorded in the M118 plan and M061/M062 authority; it does not authorize Proposal
170 policy or matrix changes.

### Dependency rule

No milestone automatically authorizes Yosemite vendoring/forking/patching, path/git overrides, parallel raw SAM, Proposal-shaped `emissary-core` APIs, or dependencies added merely to increase matrix counts.

M117 closed the ADR-0005-authorized internal Yosemite API/dependency boundary, and M118
closed the neutral variance/backup runtime prerequisite. M111 consumed both prerequisites
and is now closed; no raw/parallel SAM stack or Proposal-shaped core API is authorized.

## 4. Cross-cutting invariants

All remaining work MUST preserve:

- exact pinned wire names/types/actions/presence semantics;
- API 1-only negotiation;
- no fabricated state or accept-inert configuration;
- every `apply` cell changes real runtime behavior;
- matrix counts are evidence, not goals;
- Java implementation machinery is not automatically a portable requirement, but cited/reference semantics resolve underspecified option behavior when required by the existing planning authority;
- startup/control-plane/configuration owners remain explicit;
- feature-disabled and feature-compiled/runtime-disabled startup isolation from M115;
- direct I2P proxy traffic never falls through to clearnet DNS;
- clearnet proxy traffic requires explicit I2P outproxy;
- trusted Yosemite-derived remote peer identity;
- Streamr payload forwarding only from the intended producer;
- literal-loopback/local-target confinement from M090/M093;
- bounded transactional server admission;
- HTTP/IRC/Streamr resource/filter/anonymity protections;
- secret/key/path redaction/confinement;
- managed I2PControl key protections from M107/M108;
- no silent LeaseSet security downgrade;
- generation isolation and bounded cancellation;
- no lock across unrelated network I/O, sleeps, joins, cancellation waits or filesystem synchronization;
- external interaction remains read-only/internal-only.

## 5. Explicit non-goals

This workstream MUST NOT implement unrelated base methods, API 2, non-Proposal tunnel fields/types/actions, startup config rewrites, router-global key/profile/plugin/session/resolver machinery, unrelated protocol features, relaxed AddressBook/path confinement, weakened local-target/proxy/LeaseSet security, frontend coupling, hosted CI/fuzz/release infrastructure, or upstream interaction.

## 6. Current residual/corrective partition

M095/M105 plus M110's completion ledger are current cell evidence, subject to M116 correction.

| Owner | Families | Pre-M116 count | Current status |
|---|---|---:|---|
| M110 | `Shared`, `NewDest`, `PersistentClientKey`, `PrivKeyFile` | 31 promoted cells | historically closed; M116 corrective authority |
| M111 | `UseSSL`, `TunnelVariance`, `TunnelBackupQuantity`, `SigType`, `CustomOptions` | 4 blocked after 40 applied | closed; UseSSL remains blocked with exact semantic reason |
| M112 | proxy/plugin/jump + client `ConnectDelay`/`Profile`/remaining `DelayOpen`/`Reduce*`/`Close*`/`NewDest` | 45 blocked | closed as blocked; 24 TCP lifecycle cells applied, Streamr and unsupported owner cells retained |
| M113 | server presentation/address-routing/LeaseSet | 21 blocked | closed as blocked; corrected SAM transport at Y004 `c2db73d` (M122), still no safe TLS/multihoming/LeaseSet router primitive |

Current blocked count is 70 = 4 + 45 + 21.

M116 returned M110 cells to `blocked_primitive` where exact semantics were absent. In particular:

- all seven `NewDest` cells moved to M112 because correct semantics require M112's close-on-idle/resume primitive;
- `Shared × streamrclient` returns to blocked if safe producer-identity isolation cannot be achieved within the existing I2PControl owner.

M116 closure computes and records the exact count in `plans/closure/i2pcontrol-proposal-170/116-closure.md`. A cell may become `not_applicable` only with affirmative pinned/reference evidence.

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
M110 shared session + destination/key ownership     [CLOSED — HISTORICAL]
  |
  v
M116 M110 shared/session/NewDest corrective         [CLOSED]
  |
  v
M117 internal Yosemite fork + adapter                [CLOSED]
  |
  v
M118 neutral SAM tunnel-pool variance/backups       [CLOSED]
  |
  v
M111 SAM session-wire options                       [CLOSED — 40 APPLY / 4 UseSSL BLOCKED]
  |
  v
M112 client proxy/session-lifecycle residuals       [CLOSED AS BLOCKED — 24 APPLY / 45 REMAIN]
  |
  v
M113 server presentation + LeaseSet residuals       [CLOSED AS BLOCKED — 0 APPLY / 21 REMAIN; 82368ea]
  |
  | zero applicable residual cells + no open corrective
  v
M114 live/reference interoperability + reclosure    [CLOSED AS BLOCKED — 70 residual cells; reference/public evidence unavailable]
```

M110-M114 were numbered before later correctives M115/M116. Identifiers remain stable; execution order changes without renumbering.

## 8. Historical M109/M115 startup lifecycle line

M109 plan/closure and M115 plan/closure remain authoritative historical evidence for startup-configured tunnel action/lifecycle behavior.

M115 is closed. Runtime `i2pcontrol.enabled=false` uses historical startup managers even in a feature-capable binary; enabled runtime retains named lifecycle/mixed `All=true`; lifecycle observation is snapshot-truthful; startup shared-client session creation is retryable and final-member bounded.

M116 does not modify this line.

## 9. Historical M110 capability line

Plan: `plans/implementation/i2pcontrol-proposal-170/110-shared-client-session-and-destination-key-ownership-completion.md`

Closure: `plans/closure/i2pcontrol-proposal-170/110-closure.md`.

M110 added a bounded I2PControl-local shared-session registry, persistent/generated client destination owner, confined `PrivKeyFile` import, and server import handoff. The closure promoted 31 cells from blocked to apply.

Post-closure review does not rewrite M110 history. M116 is the separately numbered corrective authority for whether those promoted cells remain supported.

## 10. M116 — post-M110 correctness/security corrective

Plan: `plans/implementation/i2pcontrol-proposal-170/116-m110-shared-session-and-newdest-corrective-pass.md`

Status: **closed**; closure: `plans/closure/i2pcontrol-proposal-170/116-closure.md`.

M116 requires:

- linearizable lost-wakeup-free shared stream/datagram acquisition;
- cancellation-safe in-flight creator reservations;
- collision-safe, secret-redacted compatibility equality;
- bounded final-member teardown/retry behavior;
- producer-isolated Streamr application routing using trusted Yosemite peer identity;
- direct pinned/reference `NewDest` semantic freeze;
- transfer of `NewDest` to M112 if close-on-idle/resume semantics are required;
- no raw private-destination `Debug` surfaces;
- exact post-corrective matrix/ledger reconciliation.

Authorized production paths are exclusively under `emissary-cli/src/i2pcontrol/**` and are enumerated in the M116 plan. No core/util/startup/Cargo/Yosemite/frontend/workflow path is authorized.

## 10a. M117 — internal Yosemite fork and adapter integration

Plan: `plans/implementation/i2pcontrol-proposal-170/117-internal-yosemite-fork-pin-and-i2pcontrol-adapter-integration.md`.

Status: **closed**; implementation commit `22c893a`; closure:
`plans/closure/i2pcontrol-proposal-170/117-closure.md`.

M117 pins Yosemite Y001/Y002's reviewed implementation revision
`8026f5b424fc178d683e63555335f8b33e0aba04` behind the optional `i2pcontrol`-owned
`yosemite-i2pcontrol` alias. It routes I2PControl Yosemite use sites through the alias,
connects the generic session-wire and signature-aware destination APIs, and amends M062
containment evidence. It does not promote Proposal cells or implement M118 router behavior.

## 10b. M118 — neutral SAM tunnel-pool capability

Plan: `plans/implementation/i2pcontrol-proposal-170/118-neutral-sam-tunnel-pool-variance-backup-capability.md`.

Status: **closed**; implementation commit `e7f3e04`; closure:
`plans/closure/i2pcontrol-proposal-170/118-closure.md`.

M118 adds only the neutral generic SAM/tunnel-pool behavior required for signed
length variance and separately maintained standby tunnel capacity. It preserves the
existing 1..7 inbound and 1..8 outbound hop boundaries, does not change M095 counts,
and does not authorize Proposal policy in core.

## 11. M111 — SAM session-wire option completion

Plan: `plans/implementation/i2pcontrol-proposal-170/111-sam-session-wire-option-completion.md`

Status: **closed**; 40 SessionWire cells are applied through the accepted Yosemite serializer; four UseSSL cells remain blocked with an exact semantic reason.

Target: up to 44 cells. M111 applied the 40 applicable SessionWire cells and kept the four UseSSL cells blocked. No raw SAM construction, vendored/path Yosemite, or Proposal-shaped core seam was authorized.

M117 satisfied the accepted generic API/dependency part of this gate and M118 satisfied
the neutral runtime part. `UseSSL` remains blocked because its Proposal local
application/session TLS semantics are distinct from Yosemite SAM-control TLS.
Closure: `plans/closure/i2pcontrol-proposal-170/111-closure.md`.

## 12. M112 — client proxy/session-lifecycle residuals

Plan: `plans/implementation/i2pcontrol-proposal-170/112-client-proxy-and-session-lifecycle-residual-completion.md`

Status: **closed as blocked**.

Current result: 24 cells applied across six TCP client families; 45 cells remain
blocked with exact proxy/plugin/TLS-jump, profile, reduction, and Streamr
lifecycle reasons. M112 owned the `Close*`/idle lifecycle family and the seven
`NewDest` cells transferred by M116 because the correct trigger depends on that
lifecycle owner.

M111 closed the final client session configuration dependency. M112 closure:
`plans/closure/i2pcontrol-proposal-170/112-closure.md`.

## 13. M113 — server presentation/address-routing/LeaseSet residuals

Plan: `plans/implementation/i2pcontrol-proposal-170/113-server-presentation-address-routing-and-leaseset-residual-completion.md`

Status: **closed as blocked**.

Current result: 0 cells applied; 21 cells remain blocked with exact presentation/routing and LeaseSet primitive reasons. `AllowInternalSSL`, `UniqueLocalAddressPerClient`, and `MultiHoming` remain blocked because no bounded TLS termination or safe per-client/multihomed routing owner exists without weakening M093 loopback confinement; `EncryptLeaseSet`, `OptionalLookup`, and `LeaseSetClientAuths` remain blocked because, although Yosemite Y004's canonical generic encrypted-LeaseSet/client-auth fields were adopted by M122 and Y005 auth-mode consistency was adopted by M124, no Proposal path maps them and no router encrypted-LeaseSet construction owner exists. Closure: `plans/closure/i2pcontrol-proposal-170/113-closure.md`; dependency updates: `plans/closure/i2pcontrol-proposal-170/122-closure.md` and `plans/closure/i2pcontrol-proposal-170/124-closure.md`.

M124 closed the exact Y005 optional dependency adoption without promoting any Proposal cell.
The focused M113 capability/crypto-ownership audit is authorized to begin as read-only planning
work; no M113-successor implementation plan is registered until it freezes an exact LeaseSet
type/crypto/owner/lifecycle/runtime contract.

## 14. M114 — final live/reference reclosure

Plan: `plans/implementation/i2pcontrol-proposal-170/114-full-proposal-170-live-interoperability-and-final-reclosure.md`

Status: **proposed / blocked**.

Readiness requires:

- M116 closed;
- M111-M113 closed as applicable;
- M095 zero applicable blocked/planned/unsupported/unknown/inert cells;
- M105/current reconciliation zero unresolved applicable residuals;
- no open high/medium Proposal-170-scoped security/correctness corrective.

M114 implements no missing feature. It re-verifies canonical inventory, full feature suite, startup/control-plane lifecycle, all twelve data planes, reference interoperability, bounded public/reseeded truthfulness, security/anonymity, failure/recovery and changed-path containment.

Only a successful M114 closure may state `full Proposal 170 support against pinned revision 2026-05-20`. This is an internal pinned-revision claim, not upstream certification.

## 15. M116 security/concurrency requirements

### Shared registry

- one creator per compatibility key;
- waiters cannot miss success/failure/cancellation state transitions;
- creator cancellation cannot strand `creating=true`;
- no lock across Yosemite session construction/network I/O;
- bounded entries/members;
- final lease tears down owner;
- retries after failure/cancellation remain possible.

### Compatibility identity

- different persistent identities never share;
- equality is collision-safe, not a 64-bit probabilistic fingerprint;
- secret material is redacted/non-printable;
- all translated session/security settings participate in compatibility.

### Streamr

- Yosemite-derived peer identity is authoritative;
- each member forwards only its configured producer;
- another shared member's producer is isolated;
- unrelated peers are dropped;
- existing bounded subscription/refresh/payload/channel limits remain.

### NewDest

- no approximate "new identity on every start" unless pinned/reference authority proves that is the exact portable contract;
- if close-on-idle/resume is required, return cells to blocked and transfer to M112;
- `PersistentClientKey` interaction is explicit and fail-closed.

### Secret store

- private material remains confined/owner-only;
- no raw secret `Debug`/`Display`;
- no arbitrary import path expansion;
- stage/commit/discard rollback remains transactional.

## 16. Persistence, failure, cancellation and contention

Remaining work preserves the TunnelManager transaction model: validate before allocation/publication; failed edits preserve last-known-good state where promised; per-name lifecycle is serialized; tasks are generation-local/cancellable; cleanup is bounded; unsupported options fail before allocation; no partial configuration is returned as success.

M116 additionally requires creator-reservation RAII/equivalent cancellation safety, deterministic waiter wake/retry, exact identity compatibility, and Streamr producer routing before local delivery.

AddressBook/TLS/startup owners are regression scope only.

## 17. Verification policy

Use focused deterministic regressions plus existing feature-gated containment/matrix/live-runtime suites. Do not create new hosted orchestration.

Baseline commands:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

M114 additionally runs core/reference/public-network evidence only when it becomes ready.

Known stable/nightly rustfmt mismatch is a tooling limitation; record it and avoid unrelated churn.

## 18. Planning/registration discipline

Per `plans/003-planning-process.md`:

- M110 remains historically closed;
- M116 and M117 were closed historical handoffs;
- M118, M111, M112, and M113 are closed; M114 has completed its final reclosure and is closed as blocked;
- M114 is closed as blocked by its final reclosure record; no subsequent Emissary plan is dependency-ready;
- M116 closure decides exact current matrix/residual ownership;
- blocked plans may not execute because their files exist;
- material deviations require plan/ADR correction before code changes;
- closure records, not implementation assertions, decide completion.

## 19. Risks and deferred decisions

### NewDest lifecycle ownership

The largest semantic decision in M116 is whether Proposal 170 permits a standalone `NewDest` trigger or inherits reference close-on-idle/resume behavior. If the latter, M112 is the correct owner. Reblocking is a valid successful M116 outcome.

### Streamr canonical producer identity

If safe comparison requires a canonicalization capability not present in existing I2PControl/Yosemite/address-book owners, `Shared × streamrclient` must return to blocked rather than introducing a router-global resolver or accepting untrusted textual equivalence.

### Yosemite capability

M111 may retain individual cells as blocked if the accepted dependency cannot express
their exact session-wire fields. This is preferable to an unauthorized fork/vendor/parallel stack.

### Java-specific mechanisms

M112/M113 must continue separating portable contract effects from Java implementation machinery. Positive evidence is required for `not_applicable` reclassification.

### LeaseSet security

If no accepted primitive can provide encryption/client-auth semantics, M113 remains blocked and M114 cannot claim full support. No downgrade workaround is acceptable.

### Token lifetime / filesystem race residuals

Long-lived bearer tokens and the operator-controlled-base-directory path-race assumption remain separate hardening considerations absent new normative/architecture direction. They do not authorize scope expansion in M116.

## 20. Final completion rule

The roadmap closes only with successful M114 closure proving:

- exact pinned Proposal 170 inventory;
- M116 corrective closed;
- zero applicable residual option gaps;
- correct visible tunnel action semantics;
- production/live/reference interoperability;
- no high/medium Proposal-scoped security/correctness defects;
- minimal explained non-I2PControl production seams;
- feature-disabled and runtime-disabled isolation;
- internal-only external interaction.

Until then, official status remains **partial Proposal 170 support**.
