# I2PControl Proposal 170 Residual Primitive Completion Roadmap

Status: **active / partial; M142 closed as complete; no registered successor (M143 deferred pending amendment)**

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Current qualification authority:

- M139 closure: `plans/closure/i2pcontrol-proposal-170/139-closure.md`.

Planning baseline:

- repository head at M142 closure: implementation plus closure in a single commit (see `142-closure.md` for SHA);
- current M095 matrix after M142: `329 apply / 36 blocked_primitive / 475 not_applicable` across 840 TunnelManager option/family cells;
- M131 remains the historical residual applicability/primitive authority, superseded for seven cells by M140 and for four cells by M141/M142;
- M135/M136/M137/M134 session-lifecycle line is closed as complete;
- M139 is the current whole-implemented-subset runtime/security qualification authority;
- M140 is closed as complete as the residual streaming applicability authority with zero promotions;
- M141 is closed as complete as the HTTP unique-local source-address completion with 2 promotions;
- M142 is closed as complete as the HTTP SSLProxies + JumpList completion with 2 promotions.

Pinned external authority:

- I2P Proposal 170 revision `2026-05-20`, status Open;
- Java I2PControl Proposal-170 reference head `45bb593000408071dd376b78848fdc246dccd964`;
- Java I2P/I2PTunnel reference snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`;
- Yosemite optional exact revision `59140a2277bf296928d2e8ce39a148182eeff044`.

All external repositories and specifications are read-only evidence. Writes authorized by this roadmap remain internal to `eggstack/emissary`; Yosemite changes require a separately registered Yosemite plan under ADR-0005.

## 1. Purpose

Close the remaining Proposal-170 TunnelManager primitive gaps in an order that maximizes I2PControl-local implementation, minimizes changes to the heavily reviewed upstream Emissary core, and never upgrades parser/persistence/wire reachability into a support claim without an observable runtime effect.

This roadmap deliberately separates:

1. applicability correction;
2. I2PControl-local application/data-plane features;
3. small neutral lower-layer transport/session primitives;
4. security-sensitive destination/LeaseSet cryptographic infrastructure;
5. final whole-surface requalification.

The line must remain truthful if any late cryptographic primitive proves too large or unsafe for the accepted containment budget. A blocked milestone is preferable to approximate support.

## 2. Canonical authority and invariants

Read and preserve, in order:

1. `plans/000-long-term-specification.md`;
2. `plans/001-terminology-and-domain-model.md`;
3. `plans/002-long-term-roadmap.md`;
4. `plans/003-planning-process.md`;
5. ADR-0001 through ADR-0005;
6. M061/M062 containment/dependency authority;
7. M093 tunnel security;
8. M095 machine-readable support matrix;
9. M105 residual audit and M110 completion ledger;
10. M121 SigType truthfulness correction;
11. M130/M131 historical qualification/residual authority;
12. M135/M136/M137/M134 lifecycle closures;
13. M139 current integrated qualification closure;
14. this roadmap and the one currently registered implementation plan.

Cross-cutting invariants:

- Proposal-specific business/admin/application policy stays under `emissary-cli/src/i2pcontrol/**` wherever a truthful implementation is possible there.
- A non-I2PControl production seam must be neutral, owned by the canonical lower layer, exact-path authorized before implementation, and no broader than the observable primitive actually required.
- `emissary-core` must not contain Proposal-170 names, JSON-RPC concepts, TunnelManager policy, or control-plane persistence semantics.
- Every supplied unsupported value fails before allocation/effect; no accept-inert, silent coercion, fallback, or fabricated state.
- No direct-I2P-to-clearnet DNS/network fallback is introduced by proxy, TLS, plugin, or address-helper work.
- Existing literal-loopback/local-target confinement is not weakened.
- Secret material, private destinations, TLS keys, LeaseSet authorization keys, and passwords never enter diagnostics or response-facing raw config.
- State, queues, timers, registries, caches, connection tasks, and retries remain bounded and generation/cancellation safe.
- No lock spans unrelated network/filesystem I/O, sleeps, joins, DNS, TLS handshakes, or cryptographic work.
- M061/M062 exact-path/dependency evidence must be amended before closure for every actual non-policy production path; planning-only entries never authorize production.
- Yosemite remains the sole accepted SAM implementation; no parallel raw SAM stack, global patch, path override, vendoring, or floating fork is allowed.
- Historical closure records remain unchanged.
- External/upstream interaction is read-only.

## 3. Starting residual inventory

M095 reported 47 blocked cells at M140 registration; M140 re-freezes seven to `not_applicable`, leaving 40 blocked cells; M141 promotes two to `apply`, leaving 38 blocked cells; M142 promotes two to `apply`, leaving 36 blocked cells:

| Cluster | Cells | Current blocker (after M140) |
|---|---:|---|
| `SigType` | 10 | generalized destination signing/key generation |
| encrypted/authenticated LeaseSets | 15 | LeaseSet2/blinding/lookup/auth crypto + NetDB + key custody |
| streaming `Profile` (retained `client` only; M140 re-froze six cells as N/A) | 1 | actual streaming max-window/config consumer for `client` |
| presentation `UseSSL` | 4 | local application endpoint TLS identity/trust owner |
| `UseOutproxyPlugin` | 4 | real bounded local outproxy provider abstraction |
| HTTP `SSLProxies` + `JumpList` | 2 | HTTP-only TLS-outproxy/address-helper behavior |
| `UniqueLocalAddressPerClient` | 2 | deterministic per-client loopback source binding (M141 closed as complete) |
| `MultiHoming` / `shouldBundleReplyInfo` | 2 | outbound reply-LeaseSet bundling policy |
| **Total** | **40** | |

M140 mechanically re-derived these cells from M095 (`325/40/475`). M141 promotes the two `UniqueLocalAddressPerClient` cells to `apply` (`327/38/475`); M142 promotes the two HTTP-client `SSLProxies`/`JumpList` cells to `apply` (`329/36/475`). The pre-M140 47-cell table (with `Profile` × 7 and Streamr `ConnectDelay` × 1) is retained in history via M140 closure; the pre-M141 40-cell table is retained via M141 closure; the pre-M142 38-cell table is retained via M142 closure; the table below is the current-head authority after M142.

## 4. New reference evidence driving this roadmap

### 4.1 Applicability must be corrected before primitive work

The pinned Java runtime shows that several tunnel classes override or do not consume generic streaming settings:

- `I2PTunnelHTTPClientBase` forces the HTTP/CONNECT client streaming behavior and removes the interactive `maxWindowSize` override;
- `I2PTunnelIRCClient` independently forces the same bulk profile behavior;
- `I2PSOCKSTunnel` does the same and `I2PSOCKSIRCTunnel` inherits it;
- Streamr client is a UDP/datagram `StreamrConsumer` / `I2PTunnelUDPClientBase`, not an I2P streaming socket;
- generic `client` remains the candidate family where an interactive/max-window profile can be meaningful.

Similarly, Streamr has no streaming SYN/connect-delay event corresponding to `i2p.streaming.connectDelay`. M140 re-froze `Profile` × seven client families plus `ConnectDelay:streamrclient` against actual constructor/runtime consumption, retaining only `Profile:client` as blocked and reclassifying seven cells to `not_applicable` with affirmative evidence (closure `plans/closure/i2pcontrol-proposal-170/140-closure.md`).

M140 changed only `blocked_primitive -> not_applicable` with affirmative pinned evidence. It made zero `apply` promotions.

### 4.2 I2PControl-local work should be exhausted next

`UniqueLocalAddressPerClient` has exact pinned Java behavior and fits the existing accepted-stream HTTP server owner. `TrustedPeerIdentity` already exposes the canonical remote Destination hash before local-target connection, so no core identity primitive is needed.

HTTP `SSLProxies` and `JumpList` are HTTP-client presentation/routing features owned by the I2PControl HTTP proxy backend, not router primitives.

`UseSSL` is application/local endpoint TLS, distinct from M129 management TLS and Yosemite SAM-control TLS. Existing `i2pcontrol` feature dependencies already include `tokio-rustls`, `rustls-pemfile`, and `rcgen`, so the preferred implementation stays feature-owned in `emissary-cli` with no new dependency unless a later plan amendment proves otherwise.

### 4.3 Neutral core work is delayed until it is unavoidable

`Profile` may require a neutral streaming configuration seam consumed by the actual SAM streaming manager.

`MultiHoming` is not host-interface routing. The pinned Java router maps it to `shouldBundleReplyInfo`, controlling outbound reply LeaseSet bundling. Any Emissary implementation must therefore live at the neutral outbound client-message/LeaseSet owner and cannot be satisfied by serializing an I2CP option alone.

### 4.4 Crypto/LeaseSet work is staged behind explicit infrastructure gates

M121 proved that accepting only fixed Ed25519 type 7 is not truthful configurable `SigType` support. A neutral generalized signing/key-generation primitive must close before Proposal `SigType` can promote.

The 15 LeaseSet cells are exactly:

- `EncryptLeaseSet` × five server families;
- `OptionalLookup` × five server families;
- `LeaseSetClientAuths` × five server families.

They are staged in dependency order: confidentiality/publication first, lookup/blinding next, client authorization last. A generic Yosemite field does not count as an Emissary runtime primitive.

## 5. Dependency graph

```text
M139 current integrated qualification                      [CLOSED]
  |
  v
M140 streaming applicability re-freeze                     [CLOSED AS COMPLETE — 325/40/475, ZERO PROMOTION]
  |
  v
M141 UniqueLocalAddressPerClient                           [CLOSED AS COMPLETE — 327/38/475, 2 PROMOTIONS]
  |
  v
M142 HTTP SSLProxies + JumpList                            [CLOSED AS COMPLETE — 329/36/475, 2 PROMOTIONS]
  |
  v
M143 retained streaming Profile runtime (Profile:client × 1, frozen by M140) [DEFERRED; amendment with exact neutral files still required before registration]
  |
  v
M144 application/presentation UseSSL                       [DEFERRED]
  |
  v
M145 reply LeaseSet bundling / MultiHoming                 [DEFERRED]
  |
  v
M146 local outproxy provider / UseOutproxyPlugin           [DEFERRED]
  |
  v
M147 neutral destination signature-suite primitive         [DEFERRED / ZERO PROMOTION]
  |
  v
M148 Proposal SigType completion                           [DEFERRED]
  |
  v
M149 encrypted LeaseSet runtime + EncryptLeaseSet           [DEFERRED]
  |
  v
M150 OptionalLookup / blinded lookup completion             [DEFERRED]
  |
  v
M151 LeaseSetClientAuths completion                         [DEFERRED]
  |
  v
M152 final residual whole-surface requalification           [DEFERRED / ZERO PROMOTION]
```

The ordering is intentionally conservative. A later milestone may be made interface-ready in parallel, but only the next hard-dependency-ready plan is registered.

## 6. Milestones

### M140 — residual streaming applicability re-freeze

Status: **closed as complete** (closure `plans/closure/i2pcontrol-proposal-170/140-closure.md`).

Class: invariant / qualification.

Objective: adjudicate the eight suspect cells (`Profile` × seven client families and `ConnectDelay:streamrclient`) using the pinned Proposal, Java I2PControl creator/parser, actual Java I2PTunnel constructor/runtime consumption, Yosemite wire behavior, and current Emissary data planes.

Production changes: forbidden (none made).

Promotion budget: zero (zero made).

Matrix effect: seven evidence-backed `blocked_primitive -> not_applicable` reclassifications; `Profile:client` retained as the exact M143 target set. Final matrix `325/40/475`.

Exit (met): every candidate has a source-cited runtime applicability verdict, M095/counts/docs are mechanically reconciled, and the exact retained Profile implementation set for M143 is frozen.

### M141 — `UniqueLocalAddressPerClient`

Status: **closed as complete** (closure `plans/closure/i2pcontrol-proposal-170/141-closure.md`).

Class: capability.

Target: the two `httpserver` / `httpbidirserver` cells (both promoted).

Preferred owner: I2PControl accepted HTTP server handler.

Exact semantic target: when enabled for a literal-loopback local target, derive the reference-compatible local source address from the validated remote Destination hash and source-bind the outgoing local TCP socket before connect. Disabled behavior remains the ordinary local-target connect.

No core change is expected.

### M142 — HTTP `SSLProxies` + `JumpList`

Class: capability.

Target: the two HTTP-client-only cells.

Owner: `emissary-cli/src/i2pcontrol/backends/http_client.rs` plus existing HTTP policy helpers.

Requirements include bounded parsing, I2P-only proxy destinations, deterministic/failover behavior, no direct clearnet fallback, hostname-safe caching, and exact address-helper/error behavior for jump servers.

### M143 — retained streaming `Profile`

Status: deferred; retained set frozen by M140 as `Profile:client` × 1; amendment with exact neutral streaming files still required before registration.

Class: infrastructure + capability.

Target: only cells still `blocked_primitive` after M140 (`Profile:client` × 1).

Preferred lower owner: the existing neutral SAM/streaming manager configuration path. Core changes are permitted only for the minimum neutral max-window/config consumer required by actual streaming behavior.

No Streamr or constructor-overridden family may be promoted merely because a generic property can be serialized.

### M144 — presentation `UseSSL`

Class: capability / security.

Target: four M095 cells (`httpclient`, `connectclient`, `httpserver`, `httpbidirserver`).

Owner: I2PControl-local TLS endpoint/data-plane helpers using the already feature-owned TLS dependencies.

The plan must freeze client-listener vs server-local-target semantics, certificate/key/trust behavior, cancellation and handshake bounds, and fail-before-bind/connect behavior. M129 management TLS and Yosemite SAM TLS remain separate concepts.

### M145 — `MultiHoming` / `shouldBundleReplyInfo`

Class: infrastructure + capability.

Target: two HTTP server cells.

Lower primitive: neutral outbound client-message policy for optional reply LeaseSet bundling, with current/fresh LeaseSet truthfulness, bounded message size and no information leak beyond reference semantics.

I2PControl only maps the Proposal field after the neutral primitive is real.

### M146 — `UseOutproxyPlugin`

Class: infrastructure + capability.

Target: four client/proxy cells.

Owner: I2PControl-local bounded provider registry and an actual I2P-routed local outproxy provider boundary. A registry with no usable provider is infrastructure only and cannot promote cells.

No provider may perform direct clearnet fallback from the router/control process.

### M147 — neutral destination signature-suite primitive

Class: infrastructure / security.

Promotion budget: zero.

Objective: implement the exact destination signing/key-generation primitives required by the pinned reference value domain, including transient and persistent identity generation, certificate/type encoding, signing, verification compatibility, and LeaseSet/destination use.

This milestone may require exact-path changes in `emissary-core` crypto/primitives/destination owners. It must be registered only after a path-by-path security review and M061/M062 amendment; broad crypto-directory authorization is forbidden.

### M148 — Proposal `SigType`

Class: capability.

Hard dependency: M147 closed.

Target: ten M095 SigType cells.

I2PControl validates the exact supported reference domain, maps it to Yosemite/session creation and destination storage/import rules, and proves the selected type is reflected by the actual generated identity/signatures. Unsupported/noncanonical values never fall back to type 7.

### M149 — encrypted LeaseSet runtime + `EncryptLeaseSet`

Class: infrastructure + capability / security.

Target: five `EncryptLeaseSet` server cells.

Objective: add the minimum neutral LeaseSet confidentiality/publication runtime required by the pinned modes, including any required LS2/blinded construction, keys, serialization, publication and restart-safe custody. No lookup/client-auth claim is bundled merely because shared primitives exist.

### M150 — `OptionalLookup`

Class: infrastructure + capability / security.

Hard dependency: M149 closed.

Target: five `OptionalLookup` server cells.

Objective: implement exact blinded/lookup-secret derivation, NetDB lookup/decrypt policy, bounded negative caching/retry behavior and secret custody, then map the Proposal option.

### M151 — `LeaseSetClientAuths`

Class: infrastructure + capability / security.

Hard dependency: M150 closed.

Target: five `LeaseSetClientAuths` server cells.

Objective: implement the exact reference client-authorization modes (including PSK/DH where required), bounded entry validation, import/generation/storage, publication/decryption interoperability, restart behavior and redaction.

### M152 — final residual Proposal-170 requalification

Class: invariant / qualification.

Promotion budget: zero.

Objective: mechanically prove zero applicable `blocked_primitive` cells against the pinned Proposal revision, re-run M127-M139 security/runtime qualification on the final head, re-prove M061/M062 containment, and publish the final support status only if every applied cell has real runtime evidence and no high/medium Proposal-scoped defect remains.

## 7. Exact-path discipline

Every implementation plan names an initial path budget. The implementation agent must record the actual changed paths and amend M061/M062 before closure. The following are constraints, not pre-authorization:

- M140/M152: planning/tests/docs only; production changes are stop conditions.
- M141/M142/M144/M146: remain under `emissary-cli/src/i2pcontrol/**` unless a concrete impossibility is proven.
- M143: only neutral SAM/streaming config owners may cross into core.
- M145: only the neutral outbound client-message/LeaseSet bundling owner may cross into core.
- M147/M149/M150/M151: security-sensitive core/crypto/NetDB paths require explicit exact-file authorization before each milestone is registered; no prefix/glob waiver.

The roadmap itself does not expand M061's production allowlist.

## 8. Failure, cancellation, restart and contention policy

All capability plans must specify and test:

- validation before listener/session/socket/file/key allocation;
- generation-local cancellation and stale-task isolation;
- explicit timeout bounds around local connect, TLS handshake, provider selection, NetDB lookup and cryptographic waits where applicable;
- rollback/last-known-good behavior for edit/restart failures;
- no persistent state committed before a runtime generation is accepted unless the contract explicitly requires durable configuration before start;
- bounded cache/registry/auth-entry cardinality and deterministic eviction or fail-closed behavior;
- no lock held across I/O or cryptographic work;
- restart-safe secret/key ownership where a capability persists identity material.

## 9. Verification baseline

Every implementation milestone inherits, at minimum:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Milestones touching core additionally run focused and broad `emissary-core` tests/checks plus no-std coverage where the changed owner participates in no-std builds. Security-sensitive milestones require deterministic fixtures and bounded reference/live interoperability evidence before promotion.

Pre-existing rustfmt stable/nightly drift must be recorded rather than normalized through unrelated files.

## 10. Registration discipline

- M140 is closed as complete; M141 is closed as complete with 2 promotions; M142 is closed as complete; no successor is registered until M143 is amended with exact neutral streaming files.
- M143-M152 are committed as deferred handoff documents and are not executable authority until their hard dependencies close and the registry promotes exactly one next plan.
- A deferred plan's path budget is design intent, not production authorization.
- M140 froze the retained Profile cell set as `Profile:client` × 1; M143 must still be amended before registration so it names the exact neutral streaming files and M140-closure baseline counts.
- If reference/security research materially changes a later primitive contract, amend the deferred plan and roadmap before registration; do not silently reinterpret it during implementation.

## 11. Final completion rule

The full-support roadmap may claim Proposal-170 completion only after M152 records:

1. 840 matrix cells mechanically accounted for;
2. zero applicable `blocked_primitive` cells;
3. every `apply` cell backed by observable runtime behavior and end-to-end evidence;
4. every `not_applicable` cell backed by affirmative Proposal/reference applicability evidence;
5. no accept-inert or fabricated support;
6. no high/medium Proposal-scoped correctness/security issue;
7. M061/M062 containment and optional-feature dependency isolation preserved;
8. exact Yosemite pin/ownership preserved;
9. external/upstream interaction remained read-only.

Until then the repository remains **partial Proposal 170 support**.