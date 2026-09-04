# I2PControl Proposal 170 — Post-M114 Corrective Roadmap

Status: **active corrective workstream; M127-M128 closed; M129 ready; M130 blocked on corrective closure**

Original corrective baseline: `feafc6a1d9650887015a01f87bf21b57a4e92085`

M123/M124 planning baseline: `045d1e8b4eba1141d2488882f99c5ce994db91a8`

M125 audit baseline: `97083896f6170962a8c9610d056e8fc2dd57646d`

M126 planning baseline: `685eeeb20f22cdd234e4649c730000d623ad4891`

Reopened post-M126 planning baseline: `9948cfd0782a3defbd5f68cf2d4523603bdc7940`

Pinned Proposal 170 revision: `2026-05-20` (Open).

Parent roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Accepted architecture:

- ADR-0001 through ADR-0005;
- M061/M062 containment;
- M093 tunnel security;
- M119-M126 historical corrective/qualification chain.

Internal dependency fork:

- `eggstack/yosemite`, governed by its own registry and ADR-0005 consumer boundary.

## 1. Purpose and ownership boundary

Resolve correctness/security/conformance defects found after M126 without rewriting historical closures, broadening Proposal 170 into unrelated base-I2PControl parity, or contaminating the security-audited Emissary router/core with administrative policy.

The reopened line owns shared control-plane behavior needed by the implemented Proposal 170 extension surface:

- API-1 authentication/token lifetime;
- JSON-RPC 2.0 envelopes, request IDs, notifications and bounded batches;
- HTTPS/TLS configuration and fail-closed remote exposure;
- a fresh integrated requalification after those concrete fixes.

Proposal-specific business/admin/application policy remains under `emissary-cli/src/i2pcontrol/**` wherever possible. Core changes are not authorized by M127-M130. A future residual-capability milestone may touch a neutral canonical owner only under a separately reviewed plan and M061/M062 path budget.

## 2. Canonical and ADR authority

Authority order:

1. `plans/000-long-term-specification.md`;
2. `plans/001-terminology-and-domain-model.md`;
3. `plans/002-long-term-roadmap.md`;
4. `plans/003-planning-process.md`;
5. ADR-0001 through ADR-0005;
6. this roadmap;
7. registered implementation plan;
8. current source evidence.

The canonical specification explicitly excludes implementing unrelated base methods such as `GetKeys`, `GetRate`, `RouterManager`, `NetworkSetting`, or `AdvancedSettings` merely to claim Proposal 170 completion. The reopened line therefore corrects only shared I2PControl behavior needed by the extension surface. It is not a general I2PControl-parity project.

## 3. Current state and corrected M126 disposition

M119-M125 are closed historical/corrective milestones. M126 is also historically closed at `9948cfd0782a3defbd5f68cf2d4523603bdc7940`, but its clean shared-control-plane qualification is no longer current authority.

Subsequent independent review found three concrete defects/operational-security gaps that M126 either missed or accepted too broadly:

### C10 — authentication tokens have no finite lifetime

`TokenService` stores token membership but no expiry state. The RPC layer defines standard `TOKEN_EXPIRED` (`-32004`) behavior, yet production validation can never produce it. Tokens remain valid until capacity eviction, explicit clear, shutdown, or restart.

This invalidates only M126's affected authentication-lifetime qualification claim. It does not invalidate the cryptographic entropy of issued tokens or unrelated Proposal capability closures.

Owner: **M127**.

### C11 — valid JSON-RPC batches are blanket-rejected (resolved)

M126 proved that top-level arrays cannot bypass authentication, but treated blanket invalid-request rejection as sufficient. That is a security check, not JSON-RPC 2.0 batch conformance.

The corrective added bounded batch cardinality (`MAX_BATCH_ELEMENTS = 32`), per-element authentication, independent errors/results, correct notification suppression, and no unbounded task fan-out.

Owner: **M128** — **closed** (`plans/closure/i2pcontrol-proposal-170/128-closure.md`, implementation `0ed60eb`).

### C12 — non-loopback bind may use a loopback-only managed certificate

Managed TLS generates an identity for `localhost`, `127.0.0.1`, and `::1`. Current configuration still permits a non-loopback bind with only a warning. A correctly validating remote client cannot authenticate the managed loopback identity for the remote endpoint, creating pressure to disable certificate verification.

The fail-closed correction is to require complete explicit certificate/key configuration for every non-loopback bind. Managed TLS remains loopback-only.

Owner: **M129** — **ready / registered** (promoted on M128 closure).

After M127-M129 close, **M130** performs a new current-head operational/security/spec requalification and supersedes M126 only for current authority. Historical M126 evidence remains intact.

## 4. Current Proposal support state

The authoritative M095 matrix remains:

`284 apply / 96 blocked_primitive / 460 not_applicable`.

The 96 residual applicable blockers remain:

- 4 `UseSSL` cells;
- 10 `SigType` cells;
- 63 client proxy/profile/reduction/lifecycle cells, including 18 `Close`/`CloseTime`/`NewDest` cells;
- 19 server presentation/routing/LeaseSet cells.

M127-M130 are shared-control-plane corrective/qualification work. They do not promote residual cells.

Full Proposal 170 status remains **partial** until genuine owners exist for every applicable residual and a later zero-residual final reclosure completes live interoperability/security evidence.

## 5. Invariants

Every reopened corrective preserves:

- no Proposal or I2P wire-protocol expansion;
- no unrelated base-I2PControl parity project;
- Proposal policy under `emissary-cli/src/i2pcontrol/**` wherever possible;
- no broad router/core refactor;
- no frontend state/lifecycle dependency;
- ordinary Yosemite remains the registry dependency for ordinary paths;
- optional `yosemite-i2pcontrol` remains exact-pinned and feature-isolated under ADR-0005;
- TLS-only production serving with no plaintext fallback;
- one unambiguous valid credential for protected requests;
- no `accept_inert`, fabricated state, silent downgrade, or success-before-commit;
- bounded body, connection, request, auth-throttle, batch and task resource ownership;
- unsupported Proposal options fail before avoidable allocation/publication/secret generation;
- AddressBook mutations remain confined and authoritative;
- server destination/key material remains secret-safe and transactionally owned;
- M123 tunnel lifecycle cancellation terminalization remains exact;
- local-target/proxy/HTTP/IRC/Streamr anonymity/security boundaries remain intact;
- historical closure records are never rewritten to conceal later findings;
- all upstream/third-party repositories and maintainer channels remain read-only.

## 6. Target architecture for the reopened line

```text
TLS HTTP POST
  -> bounded body / connection / deadline gates
  -> JSON-RPC envelope parser (single | bounded batch)
  -> per-request API-1 authentication
       -> finite opaque token lifetime
       -> valid | expired-and-removed | unknown
  -> method/domain validation
  -> production adapter/backend
  -> authoritative owner
  -> committed observation/mutation
  -> single response | bounded batch response | notification no-content
```

Remote TLS configuration boundary:

```text
loopback bind
  -> managed loopback certificate OR explicit certificate/key

non-loopback bind
  -> complete explicit certificate/key REQUIRED
  -> no managed loopback identity fallback
```

No M127-M130 production behavior belongs below the I2PControl application layer.

## 7. Dependency graph and classes

```text
M126 post-M125 requalification                 [HISTORICAL CLOSED; C10 RESOLVED BY M127, C11 RESOLVED BY M128, C12 OPEN]
M127 token-lifetime corrective                 [CLOSED]
  |
  | sequencing dependency satisfied: batch inherits corrected token authority
  v
M128 JSON-RPC batch corrective                 [CLOSED]
  |
  | sequencing dependency for linear shared-control-plane closure
  v
M129 non-loopback TLS fail-closed corrective   [READY / REGISTERED]
  |
  v
M130 post-corrective requalification           [BLOCKED; HARD DEPENDS M127-M129]
  |
  +--> no defect/new residual owner: retain partial 284/96/460
  |
  +--> concrete defect: register M131+ focused corrective
```

Dependency classes:

- M127 has no open hard dependency and is dependency-ready.
- M128 is technically local/independent but intentionally sequencing-gated on M127 closure because it shares parser/auth dispatch paths and must consume the corrected token semantics.
- M129 is technically independent but intentionally sequencing-gated after M128 to keep one active implementation handoff and one closure authority at a time.
- M130 has hard dependencies on closed M127, M128, and M129 implementations/closures.

Only M129 is registered as the current handoff under `plans/003-planning-process.md`.

## 8. M127 — finite authentication token lifetime

Plan:

- `plans/implementation/i2pcontrol-proposal-170/127-base-auth-token-lifetime-corrective.md`

Status: **closed**; closure `plans/closure/i2pcontrol-proposal-170/127-closure.md`, implementation `098c9d1`.

Primary exit conditions:

- every issued token has finite in-process validity;
- expired state is distinguishable from unknown state;
- first expired observation atomically removes the token and returns existing `-32004`;
- no protected request succeeds after expiry;
- token capacity/input remain bounded;
- no production change occurs outside `emissary-cli/src/i2pcontrol/**`;
- matrix remains unchanged.

## 9. M128 — bounded JSON-RPC batch conformance

Plan:

- `plans/implementation/i2pcontrol-proposal-170/128-json-rpc-batch-conformance-corrective.md`

Status: **closed**; closure `plans/closure/i2pcontrol-proposal-170/128-closure.md`, implementation `0ed60eb`.

Primary exit conditions (all met; see closure):

- valid non-empty batches execute;
- invalid entries produce independent invalid-request errors;
- protected entries authenticate independently;
- notifications suppress responses; all-notification batches emit no JSON-RPC body;
- over-cap batches execute zero elements;
- no implicit intra-batch token propagation or transaction semantics;
- no unbounded per-element task fan-out;
- single-request behavior remains compatible.

## 10. M129 — non-loopback managed-TLS fail-closed policy

Plan:

- `plans/implementation/i2pcontrol-proposal-170/129-nonloopback-managed-tls-fail-closed-corrective.md`

Status: **ready / registered** (promoted on M128 closure; see `plans/closure/i2pcontrol-proposal-170/128-closure.md` §11).

Primary exit conditions:

- managed TLS remains allowed only for loopback binds;
- non-loopback binds require complete explicit certificate/key paths;
- rejection occurs before listener/task/managed-file side effects;
- explicit remote TLS remains supported;
- no TLS failure falls back to plaintext;
- no automated remote SAN/trust/mTLS scope expansion.

## 11. M130 — post-corrective current-head requalification

Plan:

- `plans/implementation/i2pcontrol-proposal-170/130-post-m127-m129-corrective-requalification.md`

Status: **blocked / unregistered**.

M130 freezes the actual merged post-M129 head and requalifies:

- M127 token lifetime/error behavior;
- M128 single/batch/notification/auth/resource behavior;
- M129 loopback-managed versus non-loopback-explicit TLS boundary;
- representative AddressBook, TunnelManager, RouterInfo and ClientServicesInfo production behavior;
- M061/M062 containment and Yosemite isolation;
- active matrix/support documentation.

M130 is the only milestone in this reopened sequence that may restore a clean “current implemented subset operationally/security qualified” statement.

## 12. Residual capability work remains deferred

M127-M130 do not make the residual 96 cells dependency-ready.

Do not register residual implementations merely because their input fields serialize. A future residual plan requires:

- exact Proposal/reference semantics;
- a genuine canonical production owner;
- no-downgrade behavior;
- exact secret/key lifecycle where relevant;
- a minimal path budget;
- end-to-end runtime evidence appropriate to the capability.

Particularly high-risk LeaseSet/authentication/multihoming/signature-type work remains blocked until those conditions are met.

## 13. Cross-cutting verification requirements

Every implementation milestone must record exact focused and broad outcomes. The common broad floor remains:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Plans add focused auth/JSON-RPC/TLS/adversarial targets as appropriate. Known stable/nightly rustfmt limitations are recorded rather than normalized through unrelated churn.

## 14. Security, lifecycle, storage, and observability requirements

- Authentication state remains memory-only, bounded, finite-lived, and cleared on shutdown/restart.
- JSON-RPC batch execution is not transactional across independent elements; already committed earlier mutations are not rolled back because a later element fails.
- Batch deadlines/cancellation must not weaken each method's existing transaction/cancellation guarantees.
- TLS configuration failures happen before serving or managed-identity side effects where the configuration is invalid.
- No corrective logs expose passwords, tokens, private destination/key material, or attacker-controlled credential content.
- No new persistent schema is expected from M127-M129.
- Existing AddressBook/tunnel persistence and restart semantics must remain green at M130.
- No corrective milestone consumes single-owner router event receivers or creates frontend-observation dependencies.

## 15. Risks

Primary risks:

- expiry-race authorization after a credential should be terminally invalid;
- batch fan-out bypassing existing concurrency/request limits;
- batch notification/error edge cases changing single-request semantics;
- remote TLS hardening accidentally breaking loopback managed deployments;
- corrective work leaking generic policy into core/router paths;
- active documentation continuing to cite M126 as clean current authority after concrete defects are known.

Mitigation is the milestone decomposition above plus a new M130 integrated requalification.

## 16. Closure and successor policy

M127 closed on the `098c9d1` head with no matrix change; M128 is now the
registered handoff. Every milestone gets a separate closure record with:

- exact implementation commit(s);
- requirement-to-evidence table;
- exact verification commands/outcomes;
- failure/cancellation/restart/contention evidence;
- compatibility/migration/security review;
- changed-path containment audit;
- unresolved findings with severity;
- next-readiness disposition;
- internal-only external-interaction attestation.

M127 closure promotes M128; M128 closure promotes M129; M129 closure promotes M130. Registry updates should occur only when the predecessor closes.

M130 clean closure restores current-head qualification but does not equal full Proposal 170 completion while applicable residuals remain blocked.

Any new concrete defect found by these milestones becomes an M131+ focused corrective. Do not opportunistically broaden an active milestone.

## 17. External-interaction boundary

Writes are authorized only to `eggstack/emissary` and, under its own separately registered work, `eggstack/yosemite`. All I2P/upstream Emissary/upstream Yosemite sources and maintainer channels are read-only.

No upstream issue, pull request, review, discussion, release, submission, merge/adoption request, contribution package, patch series, or maintainer contact is authorized by this roadmap.