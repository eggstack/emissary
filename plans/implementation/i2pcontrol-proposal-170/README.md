# Proposal 170 Implementation Handoffs

Status: **partial Proposal 170 support**. Current authoritative M095 matrix: `284 apply / 96 blocked_primitive / 460 not_applicable`.

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

- RouterInfo: 43 Proposal additions / 42 available / 1 protocol-permitted neutral / 0 unavailable.
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and cross-book precedence are operational.
- All 12 canonical TunnelManager data planes and seven actions exist.
- All six ClientServicesInfo selectors are operational.
- API 1-only auth and managed-TLS hardening are operational.
- M119 corrected M118 standby-expiry/variance semantics.
- M121 demoted unsupported `SigType` and `Close`/`CloseTime`/`NewDest` semantics instead of retaining approximate support.
- M122 exact-pinned Yosemite Y004; M124 now exact-pins Yosemite Y005 `59140a2277bf296928d2e8ce39a148182eeff044` only through the optional I2PControl alias; ordinary Yosemite remains registry 0.7.0.

Full Proposal 170 support is not claimed.

## Current corrective handoff sequence

| Handoff | Status | Scope |
|---|---|---|
| M119 | closed | M118 standby expiry + variance correctness |
| M120 | historical closed; later corrective required | deterministic server preflight and ordinary server-secret transactionality |
| M121 | closed | semantic truthfulness; matrix 284/98/458 |
| M122 | closed | exact Y004 dependency adoption; transport only |
| **M123** | **closed** | M120 commit-phase cancellation/lifecycle atomicity corrective |
| **M124** | **closed** | exact Y005 dependency adoption; no Proposal mapping |
| **M125** | **closed** | focused M113 capability/crypto-ownership audit; 2 cells reclassified, no successor implementation |

Yosemite independently registers:

- **Y005 closed** at `59140a2277bf296928d2e8ce39a148182eeff044` — `eggstack/yosemite:plans/implementation/005-y004-leaseset-auth-mode-consistency-corrective.md`.

M123, M124 and M125 are closed. M125 completed the focused M113/LeaseSet capability and
crypto-ownership audit. No successor implementation plan is registered because the audit did not
freeze a safe owner and exact runtime semantics.

## M123 — server commit cancellation atomicity

Plan:

- `123-m120-commit-phase-cancellation-atomicity-corrective.md`

Later review found that M120 disarms its server-start guard before asynchronous secret/definition durability awaits. Caller cancellation may therefore release lifecycle exclusion and abandon a partially terminalized generation. The secret-store drop helper is also best-effort under `try_lock()` contention.

M123 requires one terminal outcome—fully committed or exact rollback—for fresh, replacement and existing server starts/restarts at every commit boundary. It is I2PControl-only and changes no matrix cell.

## Yosemite Y005 — auth-mode consistency

Y004's canonical property spelling and DH/PSK representation remain valid, but its typed API can currently serialize client-auth entries under a mode the Java reference does not consume.

Y005 freezes the relationship among LeaseSet type, `leaseSetAuthType`, and numbered DH/PSK entries and rejects typed combinations whose security material would be inert. It implements no router cryptography or Proposal policy.

Current Emissary has no active Proposal LeaseSet client-auth mapping, so Y005 is a prerequisite for future work rather than a currently active runtime downgrade.

## M124 — exact Y005 adoption

Plan:

- `124-y005-auth-consistency-pin-adoption.md`

M124 is closed by `plans/closure/i2pcontrol-proposal-170/124-closure.md`. It reviewed and
exact-pinned Y005 through the existing optional `yosemite-i2pcontrol` alias, updated
lock/containment evidence, and proved the corrected dependency behavior through I2PControl tests.

M124 may not implement M113 LeaseSet features or change M095 counts.

## Residual Proposal ownership

Current 96 blocked cells are:

- 4 `UseSSL` cells;
- 10 `SigType` cells;
- 63 client proxy/profile/reduction/lifecycle cells, including 18 `Close`/`CloseTime`/`NewDest` cells;
- 19 server presentation/routing/LeaseSet cells; the two server-role `AllowInternalSSL` cells are
  not applicable under Proposal 170's HTTP-client filtering classification.

M125 found that serializer reachability is transport capability only: current Emissary still has
no accepted encrypted/authenticated LeaseSet construction owner. The two server-role
`AllowInternalSSL` cells were corrected to `not_applicable`; all other M113 residuals remain
explicitly blocked before allocation.

## Containment

Preferred production ownership remains `emissary-cli/src/i2pcontrol/**`.

M061/M062 are authoritative. M123 is explicitly I2PControl-only. M124 may change only the existing optional exact-revision fork alias and I2PControl dependency evidence. No global `[patch]`, path dependency, vendoring, floating Yosemite ref, frontend coupling or broad router refactor is authorized.

## Verification baseline

Implementation plans refine focused tests, but the broad baseline remains:

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

Known stable/nightly rustfmt drift must be recorded rather than normalized through unrelated formatter churn.

## Internal-only rule

All writes remain internal to `eggstack/emissary` and, under its own registry, `eggstack/yosemite`. External I2P/upstream Emissary/upstream Yosemite sources are read-only evidence.

No plan authorizes upstream issue/PR/review/contact/submission/release/merge/adoption activity or contribution preparation.
