# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 support; M063 corrective ready

This directory contains bounded internal implementation and closure handoffs for the I2PControl Proposal 170 subsystem.

Authoritative planning references:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`
- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml`
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`

Pinned Proposal 170 revision: `2026-05-20`.

## Internal-only rule

All work is internal to `eggstack/emissary`. External specifications, reference implementations, and upstream source are read-only evidence. No plan authorizes upstream submissions or repository changes.

## Current handoff

M063 is the sole dependency-ready containment handoff:

- `063-m062-closure-and-feature-guard-corrective.md` — **ready**.

M063 is planning/test-only. It corrects stale M062 status/head records and strengthens `m062_dependency_containment.rs` so unrelated Cargo features cannot activate the I2PControl-only direct `subtle` dependency indirectly through local feature composition.

M062 planning head: `a0d9f2dcc15fdeb5fcbe6658c0399ff9c8c9575b`.

M062 implementation/closure commit and M063 planning baseline: `fac2a0cdf75e3aa805acaf976f5a1ca69da6cf2c`.

M063 may not modify Cargo manifests, `Cargo.lock`, production Rust source, runtime/core behavior, authentication behavior, or Proposal 170 capability scope.

## Current containment authority

| Handoff | Status | Disposition |
|---|---|---|
| M058 | closed | fork-delta inventory and path budgets |
| M059 | closed | original CLI/runtime containment |
| M060 | closed | core observation containment |
| M061 | closed | exact source-path boundary and static guard |
| M062 | production fix accepted; closure/evidence corrective required | dependency ownership correction and dependency authority landed at `fac2a0c` |
| M063 | ready | reconcile closure records and enforce indirect feature activation invariant |

M061 remains the source-path containment authority. `062-dependency-containment.toml` remains the dependency-policy authority. M063 strengthens the existing M062 test rather than creating a new dependency-policy layer.

## Accepted M062 production state

The following state is frozen under M063:

- root `Cargo.toml` has no I2PControl-only workspace `subtle` declaration;
- `emissary-cli/Cargo.toml` owns `subtle` locally as version `2.6.1`, `default-features = false`, `optional = true`;
- `i2pcontrol` explicitly activates `dep:subtle`;
- `Cargo.lock` is unchanged from the M062 planning baseline;
- no production Rust source changed.

## Durable dependency rule

A direct dependency whose only direct consumer is code gated by `i2pcontrol` must be optional and activated by that feature. An unrelated local feature must not activate that dependency directly or indirectly through another local feature.

This rule concerns direct dependency activation. A crate name may still appear transitively for unrelated package requirements.

## Accepted Proposal 170 state

The RouterInfo matrix remains 43 total / 37 available / 1 protocol-permitted neutral / 5 unavailable. M051 remains blocked by absent substantive news/banned-peer owners. Unsupported tunnel data planes remain out of scope.

M063 is not authorized to change any of these dispositions.

## M063 verification discipline

Use focused local checks only:

- `m062_dependency_containment`, including direct/indirect/cycle/weak-edge feature cases;
- retained `m061_containment`;
- feature-off and feature-on `emissary-cli` checks;
- exact changed-path review from `fac2a0c`;
- proof that Cargo manifests, lockfile, production source, and M061 authority remain unchanged;
- `git diff --check`.

No CI/release expansion, full workspace matrix, fuzzing, soak testing, or general dependency cleanup is part of M063.

## Historical source-completion status

M053/M045 and M046-M048 remain closed. M049/M050/M052 were corrected by M054-M056. M057 is closed planning consistency. M051 remains blocked with its accepted limitation.

## Final status rule

Containment completion does not mean full Proposal 170 support. After M063 closes, the containment workstream should return to closed with M061 governing source paths and the M062 dependency authority plus strengthened test governing direct dependency ownership. No upstream review or acceptance is implied.
