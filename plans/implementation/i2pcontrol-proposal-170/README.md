# Proposal 170 Implementation Handoffs

Status: **partial Proposal 170 support; M131 ready / registered**.

Pinned Proposal revision: `2026-05-20` (Open).

Current runtime/security qualification authority:

- M130 closure: `plans/closure/i2pcontrol-proposal-170/130-closure.md`;
- M130 implementation `fe1a981`;
- M130 closure / M131 production-behavior baseline `a68094e128d2b92f0fd5b350e38512ef6b65cb6b`.

Current registered handoff:

- `131-residual-applicability-and-primitive-architecture-refreeze.md`.

The authoritative M095 starting matrix remains `284 apply / 96 blocked_primitive / 460 not_applicable` until M131 produces evidence-backed current-authority corrections.

## Authority

Read in this order:

1. `plans/000-long-term-specification.md`;
2. `plans/001-terminology-and-domain-model.md`;
3. `plans/002-long-term-roadmap.md`;
4. `plans/003-planning-process.md`;
5. ADR-0001 through ADR-0005;
6. subsystem roadmaps;
7. `plans/registry.md`;
8. the specific registered plan.

Containment/support evidence:

- `061-containment-boundary.toml`;
- `062-dependency-containment.toml`;
- `095-full-support-matrix.toml`;
- `105-residual-option-audit.toml`;
- `110-completion-ledger.toml`.

## Current production/support state

According to M130 and preceding closures:

- RouterInfo: 43 Proposal additions / 42 available / 1 neutral / 0 unavailable;
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and cross-book precedence operational;
- all 12 TunnelManager data planes and seven actions exist for the currently claimed subset;
- all six ClientServicesInfo selectors operational;
- M121 truthfully demoted unsupported `SigType` and `Close`/`CloseTime`/`NewDest` approximations;
- Yosemite Y005 `59140a2277bf296928d2e8ce39a148182eeff044` is exact-pinned only through the optional I2PControl alias;
- M127 finite token lifetime, M128 bounded JSON-RPC batches and M129 fail-closed non-loopback TLS are closed;
- M130 requalified the corrected shared control plane and representative Proposal production.

Full Proposal 170 support is not claimed.

## M131 — current registered handoff

Plan:

- `131-residual-applicability-and-primitive-architecture-refreeze.md`

Status: **ready / registered**.

Purpose:

Re-freeze all 96 currently blocked TunnelManager cells before implementing additional lower-layer primitives. M131 is planning/evidence-only and may not change runtime production behavior.

M131 must deliver:

- exact mechanical enumeration of all 96 starting blocked cells;
- cell-level applicability and semantic evidence;
- evidence-backed blocker-owner corrections;
- `not_applicable` corrections only with affirmative pinned/reference evidence;
- **zero `apply` promotions**;
- a machine-readable residual primitive map;
- canonical owners and minimal future path budgets;
- dependency/security/failure/restart analysis for each future primitive cluster;
- one next dependency-ready M132+ handoff at closure, or an explicit no-handoff result.

Required semantic review includes:

- `UseOutproxyPlugin`, `SSLProxies`, `JumpList` applicability;
- every Streamr residual;
- `Profile` streaming-window/runtime semantics;
- session application-activity, idle reduction, close and resume semantics for `Reduce*`, `Close*`, `NewDest`;
- `UniqueLocalAddressPerClient` source-bind behavior and loopback confinement;
- `MultiHoming` versus `shouldBundleReplyInfo`;
- exact `UseSSL` type/direction/identity/trust semantics;
- `SigType` crypto/destination requirements;
- encrypted/authenticated LeaseSet runtime ownership below Yosemite Y005.

M131 explicitly authorizes no production Rust changes, no Cargo/dependency changes and no Yosemite writes.

## Current residual starting state

The 96 blocked cells are currently summarized as:

- 4 `UseSSL` cells;
- 10 `SigType` cells;
- 63 client proxy/profile/reduction/lifecycle cells;
- 19 server presentation/routing/LeaseSet cells.

M131 must derive the exact set mechanically from M095. Counts are evidence, not targets.

## Candidate future primitive clusters

M131 must test rather than assume these clusters:

- HTTP address-helper / SSL-outproxy behavior;
- local presentation TLS;
- streaming profile/window configuration;
- I2P-session activity + idle reduction/close/resume;
- per-client local source addressing;
- `shouldBundleReplyInfo` / sender LeaseSet bundling;
- outproxy-provider/plugin integration;
- destination signing-type generation;
- encrypted/authenticated LeaseSets.

No M132+ plan is currently registered.

## Recent closed sequence

| Handoff | Status | Scope |
|---|---|---|
| M121 | closed | semantic truthfulness; `SigType` and idle lifecycle demotions |
| M122 | closed | exact Y004 dependency adoption |
| M123 | closed | cancellation/commit atomicity |
| M124 | closed | exact Y005 dependency adoption |
| M125 | closed | residual capability audit; two `AllowInternalSSL` applicability corrections |
| M126 | historical | pre-corrective shared-control-plane requalification |
| M127 | closed | finite API token lifetime |
| M128 | closed | bounded JSON-RPC batch conformance |
| M129 | closed | non-loopback explicit-TLS requirement; managed loopback-only |
| M130 | closed | integrated current-head runtime/security requalification |
| M131 | **ready / registered** | residual applicability and primitive-architecture re-freeze |

Historical closure records remain unchanged.

## Canonical base-I2PControl scope

`plans/000-long-term-specification.md` excludes unrelated base-method parity from Proposal-170 completion. `GetKeys`, `GetRate`, `RouterManager`, `NetworkSetting`, `AdvancedSettings` and similar methods remain outside this workstream.

Shared base behavior is in scope only where needed for the implemented extension surface: authentication/version/token semantics, HTTPS, JSON-RPC envelopes/IDs/notifications/batches and protected dispatch.

## Containment

Preferred production ownership remains `emissary-cli/src/i2pcontrol/**`.

Any future path outside that boundary requires a neutral canonical owner, exact path budget, containment amendment and separately registered plan. No Proposal-shaped core API, global Yosemite patch, path dependency, vendoring, floating fork, frontend coupling or broad router refactor is authorized.

All external I2P/upstream Emissary/upstream Yosemite sources are read-only evidence.

## Verification baseline

M131 is planning/evidence-only. If it changes M095/M105/M110 or their focused guards, run at minimum:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --no-fail-fast
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

Existing stable/nightly rustfmt drift must be recorded rather than normalized through unrelated churn.
