# Proposal 170 Implementation Handoffs

Status: **partial Proposal 170 support; post-M126 corrective line reopened**. M127-M128 closed; M129 is the current registered handoff. The authoritative M095 matrix remains `284 apply / 96 blocked_primitive / 460 not_applicable`.

Pinned Proposal revision: `2026-05-20` (Open).

This directory contains bounded internal implementation, corrective, audit and dependency handoffs for the I2PControl Proposal 170 workstream.

## Authority

Read in this order:

1. `plans/000-long-term-specification.md`
2. `plans/001-terminology-and-domain-model.md`
3. `plans/002-long-term-roadmap.md`
4. `plans/003-planning-process.md`
5. ADR-0001 through ADR-0005
6. subsystem roadmaps
7. `plans/registry.md`
8. the specific registered implementation plan

Containment/support evidence:

- `061-containment-boundary.toml`
- `062-dependency-containment.toml`
- `095-full-support-matrix.toml`
- `105-residual-option-audit.toml`
- `110-completion-ledger.toml`

## Current production/support state

- RouterInfo: 43 Proposal additions / 42 available / 1 protocol-permitted neutral / 0 unavailable according to current closure evidence.
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and cross-book precedence are operational according to current closure evidence.
- All 12 canonical TunnelManager data planes and seven actions exist for the currently claimed subset.
- All six ClientServicesInfo selectors are operational according to current closure evidence.
- M121 truthfully demoted unsupported `SigType` and `Close`/`CloseTime`/`NewDest` semantics instead of retaining approximate support.
- M124 exact-pins Yosemite Y005 `59140a2277bf296928d2e8ce39a148182eeff044` only through the optional I2PControl alias; ordinary Yosemite remains registry 0.7.0.
- M125 corrected two server-role `AllowInternalSSL` applicability cells and left 96 blocked cells with no dependency-ready owner.
- M126 is historical current-head evidence, but subsequent review found three shared-control-plane defects/gaps; its clean auth/TLS/JSON-RPC qualification is superseded pending M129-M130 (C10 resolved by closed M127, C11 resolved by closed M128).

Full Proposal 170 support is not claimed.

## Reopened corrective sequence

| Handoff | Status | Scope |
|---|---|---|
| M127 | **closed** | finite API-1 token lifetime; expired-vs-unknown behavior; auth bounds |
| M128 | **closed** | bounded JSON-RPC batch conformance and per-element auth/resource rules |
| M129 | **ready / registered** | non-loopback bind requires explicit TLS certificate/key; managed TLS loopback-only |
| M130 | blocked / unregistered | integrated post-M127-M129 current-head requalification |

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md`

Only M129 is registered now, consistent with `plans/003-planning-process.md`. Later plan files are written to make the full corrective line explicit, but their implementation status remains gated until predecessor closure.

## M127 — closed corrective

Plan:

- `127-base-auth-token-lifetime-corrective.md`

Closure:

- `plans/closure/i2pcontrol-proposal-170/127-closure.md` (implementation `098c9d1`).

M127 corrected the concrete authentication-lifetime defect missed by M126:

- issued opaque tokens gained finite one-day monotonic in-process validity;
- validation distinguishes valid, expired-and-removed and unknown credentials;
- expired lookup maps to existing `-32004 TOKEN_EXPIRED`;
- later use of the removed token maps to existing `-32003 INVALID_TOKEN`;
- token capacity/input, conflict rejection, throttle, shutdown clearing and secret safety remain bounded;
- no production change outside `emissary-cli/src/i2pcontrol/**` occurred;
- no Proposal matrix cell changed.

M127 does not add unrelated base I2PControl methods.

## M128 — closed corrective

Plan:

- `128-json-rpc-batch-conformance-corrective.md`

Closure:

- `plans/closure/i2pcontrol-proposal-170/128-closure.md` (implementation `0ed60eb`).

M128 started from the closed-M127 head and inherited the corrected
token-lifetime semantics. It replaced blanket top-level-array rejection
with bounded JSON-RPC 2.0 batch behavior:

- non-empty batch support (`MAX_BATCH_ELEMENTS = 32`);
- per-element authentication and independent result/error handling;
- exact notification suppression and no-content all-notification behavior;
- explicit batch cardinality bound and zero execution for over-cap batches;
- no implicit token propagation or cross-element transaction semantics;
- no unbounded task fan-out;
- unchanged single-request method/domain behavior.

M128 supersedes only M126's affected batch-conformance qualification
claim. Historical M126/M127 closures remain unchanged.

## M129 — current registered handoff

Plan:

- `129-nonloopback-managed-tls-fail-closed-corrective.md`

Status: **ready / registered** (promoted on M128 closure).

M129 makes the managed self-signed identity explicitly loopback-only.
It starts from the closed-M128 head.

## M129 — queued remote TLS fail-closed corrective

Plan:

- `129-nonloopback-managed-tls-fail-closed-corrective.md`

M129 is written and registered as the current handoff after M128. It makes the managed self-signed identity explicitly loopback-only:

- loopback bind may use managed or explicit TLS material;
- non-loopback bind requires complete explicit certificate + private key paths;
- invalid remote/managed configuration fails before listener/task/managed-file side effects;
- explicit remote TLS remains supported;
- no automatic SAN discovery, CA/trust management, mTLS or plaintext fallback is added.

## M130 — blocked integrated requalification

Plan:

- `130-post-m127-m129-corrective-requalification.md`

M130 hard-depends on M127-M129 closure. It freezes the actual merged post-M129 head, mechanically recomputes matrix authority, black-box requalifies the corrected auth/JSON-RPC/TLS boundary, reruns representative AddressBook/TunnelManager/RouterInfo/ClientServicesInfo production evidence, and re-audits M061/M062 containment and Yosemite isolation.

M130 is the only milestone in this sequence allowed to restore a clean current-head “implemented subset operationally/security qualified” statement.

## Canonical base-I2PControl scope

`plans/000-long-term-specification.md` explicitly excludes implementing unrelated base methods such as `GetKeys`, `GetRate`, `RouterManager`, `NetworkSetting`, or `AdvancedSettings` merely to claim Proposal 170 completion.

The reopened line therefore fixes only shared base behavior required by the extension surface: authentication/version/token semantics, HTTPS serving, JSON-RPC envelopes/IDs/notifications/batches, and protected dispatch. It is not a general I2PControl-parity project.

## Historical corrective sequence

| Handoff | Disposition |
|---|---|
| M119 | closed — standby expiry + variance correctness |
| M120 | historical closed; cancellation claim later superseded by M123 |
| M121 | closed — semantic truthfulness; historical matrix 284/98/458 |
| M122 | closed — exact Y004 dependency adoption; transport only |
| M123 | closed — commit-phase cancellation/lifecycle atomicity |
| M124 | closed — exact Y005 dependency adoption; no Proposal mapping |
| M125 | closed — M113 capability/crypto audit; two cells reclassified |
| M126 | historical closed — later C10-C12 findings supersede its clean shared-control-plane qualification (C10 resolved by M127, C11 resolved by M128) |
| M127 | closed — finite token lifetime, expired/unknown mapping, bounds; matrix unchanged |
| M128 | closed — bounded batch conformance (`MAX_BATCH_ELEMENTS = 32`), per-element auth, notification/no-content rules; matrix unchanged |

Historical closure records are retained unchanged.

## Residual Proposal ownership

The 96 blocked cells remain:

- 4 `UseSSL` cells;
- 10 `SigType` cells;
- 63 client proxy/profile/reduction/lifecycle cells, including 18 `Close`/`CloseTime`/`NewDest` cells;
- 19 server presentation/routing/LeaseSet cells.

M127-M130 are correctness/conformance/security work and do not promote these residuals. Parser/serializer reachability is not capability evidence. A future residual implementation requires a genuine canonical owner, exact runtime semantics, no-downgrade proof, path budget and end-to-end evidence before registration.

## Containment

Preferred production ownership remains `emissary-cli/src/i2pcontrol/**`.

M061/M062 are authoritative. No global `[patch]`, path dependency, vendoring, floating Yosemite ref, frontend coupling or broad router refactor is authorized. M127-M130 specifically authorize no core/router production changes.

## Verification baseline

Individual plans refine focused tests, but the broad baseline remains:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-core
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Known stable/nightly rustfmt drift must be recorded rather than normalized through unrelated formatter churn.

## Internal-only rule

All writes remain internal to `eggstack/emissary` and, under its own registry, `eggstack/yosemite`. External I2P/upstream Emissary/upstream Yosemite sources are read-only evidence.

No plan authorizes upstream issue/PR/review/contact/submission/release/merge/adoption activity or contribution preparation.
