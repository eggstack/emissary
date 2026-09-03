# Proposal 170 Implementation Handoffs

Status: **partial Proposal 170 support**. M126 is closed against the current implementation head; the authoritative M095 matrix remains `284 apply / 96 blocked_primitive / 460 not_applicable`.

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
- All 12 canonical TunnelManager data planes and seven actions exist; current `apply` semantics are subject to M126 independent requalification.
- All six ClientServicesInfo selectors are operational according to current closure evidence.
- API 1-only auth and managed-TLS hardening are operational according to current closure evidence.
- M119 corrected M118 standby-expiry/variance semantics.
- M121 demoted unsupported `SigType` and `Close`/`CloseTime`/`NewDest` semantics instead of retaining approximate support.
- M122 exact-pinned Yosemite Y004; M124 exact-pins Yosemite Y005 `59140a2277bf296928d2e8ce39a148182eeff044` only through the optional I2PControl alias; ordinary Yosemite remains registry 0.7.0.
- M125 corrected two server-role `AllowInternalSSL` applicability cells and left 96 blocked cells with no dependency-ready owner.

Full Proposal 170 support is not claimed.

## Current corrective handoff sequence

| Handoff | Status | Scope |
|---|---|---|
| M119 | closed | M118 standby expiry + variance correctness |
| M120 | historical closed; later corrective required | deterministic server preflight and ordinary server-secret transactionality |
| M121 | closed | semantic truthfulness; historical matrix 284/98/458 |
| M122 | closed | exact Y004 dependency adoption; transport only |
| M123 | closed | M120 commit-phase cancellation/lifecycle atomicity corrective |
| M124 | closed | exact Y005 dependency adoption; no Proposal mapping |
| M125 | closed | focused M113 capability/crypto-ownership audit; 2 cells reclassified, no successor implementation |
| M126 | closed | current-head operational/security/spec requalification; implemented subset qualified |

Yosemite independently closed:

- **Y005** at `59140a2277bf296928d2e8ce39a148182eeff044` — `eggstack/yosemite:plans/implementation/005-y004-leaseset-auth-mode-consistency-corrective.md`.

## M126 — closed current-head operational/security/spec requalification

Plan:

- `126-post-m125-operational-security-and-spec-requalification.md`

Planning baseline:

- `685eeeb20f22cdd234e4649c730000d623ad4891`.

M126 independently requalifies the current implemented subset rather than treating historical closures as sufficient current-head proof.

It must:

- reconstruct the pinned Proposal 170 inventory and mechanically reproduce or truthfully correct `284 / 96 / 460`;
- trace every claimed `apply` family through authentication/domain validation to a real production owner;
- black-box test the auth/TLS/JSON-RPC/resource boundary;
- requalify AddressBook persistence/confinement/atomicity;
- requalify TunnelManager real-runtime lifecycle, rollback, cancellation, admission and unsupported-option fail-before-effect behavior;
- verify RouterInfo and ClientServicesInfo source truthfulness and exact wire semantics;
- re-audit M061/M062 containment;
- reconcile active documentation/evidence, including stale current-count statements.

Concrete production/security defects found during M126 require separately registered M127+ corrective plans. M126 does not opportunistically implement them and does not implement the 96 blocked cells.

M126 independently reproduced the post-M125 counts, corrected the M062 historical allowlist for M125/M126 planning evidence, and reconciled active guidance to the current matrix.

No M127+ corrective plan was required: no current-head production or security defect was found, and no blocked primitive became dependency-ready.

## Recently closed handoffs

### M125 — M113 capability/crypto ownership

Plan:

- `125-m113-capability-crypto-ownership-audit.md`

M125 is closed by `plans/closure/i2pcontrol-proposal-170/125-closure.md`. It confirmed that the remaining encrypted/authenticated LeaseSet, per-client address and multihoming/presentation-routing cells still lack a safe canonical runtime owner. The two server-role `AllowInternalSSL` cells were reclassified to `not_applicable` under Proposal 170's HTTP-client filtering classification.

### M124 — exact Y005 adoption

Plan:

- `124-y005-auth-consistency-pin-adoption.md`

M124 is closed by `plans/closure/i2pcontrol-proposal-170/124-closure.md`. It independently reviewed and exact-pinned Y005 through the existing optional `yosemite-i2pcontrol` alias, updated lock/containment evidence, and proved the corrected dependency behavior through I2PControl tests.

M124 did not map Proposal LeaseSet fields and did not change M095 counts.

### M123 — server commit cancellation atomicity

Plan:

- `123-m120-commit-phase-cancellation-atomicity-corrective.md`

M123 is closed and supersedes only M120's affected cancellation-completeness claim. It requires one terminal outcome—fully committed or exact rollback—for fresh, replacement and existing server starts/restarts at every commit boundary. It is I2PControl-only and changes no matrix cell.

## Residual Proposal ownership

Current 96 blocked cells entering M126 are:

- 4 `UseSSL` cells;
- 10 `SigType` cells;
- 63 client proxy/profile/reduction/lifecycle cells, including 18 `Close`/`CloseTime`/`NewDest` cells;
- 19 server presentation/routing/LeaseSet cells; the two server-role `AllowInternalSSL` cells are not applicable under Proposal 170's HTTP-client filtering classification.

M125 found that serializer reachability is transport capability only: current Emissary still has no accepted encrypted/authenticated LeaseSet construction owner. All remaining unsupported cells stay explicitly blocked before allocation unless M126 or a later capability audit establishes a real owner.

## Containment

Preferred production ownership remains `emissary-cli/src/i2pcontrol/**`.

M061/M062 are authoritative. No global `[patch]`, path dependency, vendoring, floating Yosemite ref, frontend coupling or broad router refactor is authorized. Any future residual implementation requires a neutral canonical owner outside I2PControl and a separately registered plan before broader production changes begin.

## Verification baseline

Implementation plans refine focused tests, but the broad baseline remains:

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

Known stable/nightly rustfmt drift must be recorded rather than normalized through unrelated formatter churn.

## Internal-only rule

All writes remain internal to `eggstack/emissary` and, under its own registry, `eggstack/yosemite`. External I2P/upstream Emissary/upstream Yosemite sources are read-only evidence.

No plan authorizes upstream issue/PR/review/contact/submission/release/merge/adoption activity or contribution preparation.
