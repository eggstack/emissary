# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 support; M062 dependency-surface containment corrective ready

This directory contains bounded internal implementation and closure handoffs for the I2PControl Proposal 170 subsystem.

Authoritative planning references:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`
- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml`

Pinned Proposal 170 revision: `2026-05-20`.

## Internal-only rule

All work is internal to `eggstack/emissary`. External specifications, reference implementations, and upstream source are read-only evidence. No plan authorizes upstream submissions or repository changes.

## Current handoff

M062 is the sole dependency-ready containment handoff:

- `062-dependency-surface-containment.md` — **ready**.

M062 corrects a narrow Cargo feature-ownership gap found after M061. The `subtle` direct dependency is used by the feature-gated I2PControl authentication implementation but is currently unconditional in `emissary-cli` and declared at workspace scope.

The implementation must:

1. verify there is no independent non-I2PControl direct workspace consumer;
2. remove the I2PControl-only workspace declaration;
3. declare `subtle` locally in `emissary-cli` as optional with default features disabled;
4. activate it explicitly from the `i2pcontrol` feature;
5. add `062-dependency-containment.toml` and `m062_dependency_containment.rs`;
6. preserve M061 source containment and all runtime behavior.

Authorized production files are only:

- `Cargo.toml`
- `emissary-cli/Cargo.toml`

`Cargo.lock` is expected to remain unchanged. Any lockfile change requires narrow review and must contain no unrelated dependency-resolution changes.

No production Rust source file is authorized for modification by M062.

## Current containment authority

M058 through M061 remain accepted closed evidence:

| Handoff | Status | Disposition |
|---|---|---|
| M058 | closed | fork-delta inventory and path budgets |
| M059 | closed | original CLI/runtime containment |
| M060 | closed | core observation containment |
| M061 | closed | current exact source-path boundary and static guard |
| M062 | ready | direct dependency feature ownership and dependency guard |

M061 remains the source-path containment authority. M062 adds a complementary dependency-surface authority; it does not rewrite M061 history.

## Durable dependency rule

A direct dependency whose only direct consumer is code gated by the `i2pcontrol` feature must itself be optional and activated by that feature. It should not be an unconditional default-CLI dependency or a workspace-level direct dependency unless another independently justified direct consumer exists.

This rule applies to direct dependency ownership. A dependency name may still appear transitively for unrelated package requirements.

## Accepted Proposal 170 state

The accepted RouterInfo matrix remains:

- 43 total additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable.

The five unavailable additions remain transit 15s, news, banned peers, and both network-error rows. M051 remains blocked by absent substantive news/banned-peer owners.

Unsupported tunnel data planes remain out of scope.

M062 may not change any of these dispositions.

## Verification for M062

Use only bounded local checks:

- direct-use and manifest ownership inspection;
- `cargo metadata --format-version 1 --no-deps`;
- `cargo check -p emissary-cli --no-default-features`;
- `cargo check -p emissary-cli --no-default-features --features i2pcontrol`;
- focused M062 dependency-containment test;
- focused I2PControl authentication tests;
- retained M061 containment test;
- exact changed-path and lockfile review;
- `git diff --check`.

Do not add verification infrastructure or broaden this into a general dependency cleanup.

## Historical source-completion status

M053/M045 and M046-M048 remain closed. M049/M050/M052 were corrected by M054-M056. M057 is closed planning consistency. M051 remains blocked with its accepted limitation.

## Final status rule

Containment completion does not mean full Proposal 170 support. After M062 closure, the containment workstream should return to closed with M061 governing source paths and M062 governing direct dependency ownership. No upstream review or acceptance is implied.
