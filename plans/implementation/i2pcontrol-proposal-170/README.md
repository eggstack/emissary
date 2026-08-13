# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 support; containment corrective sequence complete; M062 closed

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

No implementation plan is currently dependency-ready. M062 was the sole dependency-ready containment handoff and is now closed:

- `062-dependency-surface-containment.md` — **closed** (`plans/closure/i2pcontrol-proposal-170/062-closure.md`).

M062 corrected a narrow Cargo feature-ownership gap found after M061. The `subtle` direct dependency is used by the feature-gated I2PControl authentication implementation and was unconditional in `emissary-cli` and declared at workspace scope.

M062:

1. verified there is no independent non-I2PControl direct workspace consumer (the `emissary-core` declaration uses a literal version, not `workspace = true`);
2. removed the I2PControl-only root workspace declaration;
3. declared `subtle` locally in `emissary-cli` as optional with default features disabled;
4. activated it explicitly from the `i2pcontrol` feature;
5. added `062-dependency-containment.toml` and `m062_dependency_containment.rs`;
6. preserved M061 source containment and all runtime behavior.

The only production files modified by M062 are:

- `Cargo.toml`
- `emissary-cli/Cargo.toml`

`Cargo.lock` is byte-identical to the M062 planning baseline. No production Rust source file was modified by M062.

## Current containment authority

M058 through M062 are accepted closed evidence:

| Handoff | Status | Disposition |
|---|---|---|
| M058 | closed | fork-delta inventory and path budgets |
| M059 | closed | original CLI/runtime containment |
| M060 | closed | core observation containment |
| M061 | closed | current exact source-path boundary and static guard |
| M062 | closed | direct dependency feature ownership and dependency guard |

M061 remains the source-path containment authority. M062 is the complementary dependency-surface authority. Neither is rewritten by the other.

## Durable dependency rule

A direct dependency whose only direct consumer is code gated by the `i2pcontrol` feature must itself be optional and activated by that feature. It must not be an unconditional default-CLI dependency or a workspace-level direct dependency unless another independently justified direct consumer exists.

This rule applies to direct dependency ownership. A dependency name may still appear transitively for unrelated package requirements.

## Accepted Proposal 170 state

The accepted RouterInfo matrix remains:

- 43 total additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable.

The five unavailable additions remain transit 15s, news, banned peers, and both network-error rows. M051 remains blocked by absent substantive news/banned-peer owners.

Unsupported tunnel data planes remain out of scope.

M062 did not change any of these dispositions.

## Verification executed for M062

Local and proportional checks only:

- direct-use and manifest ownership inspection;
- `cargo metadata --format-version 1 --no-deps`;
- `cargo check -p emissary-cli --no-default-features`;
- `cargo check -p emissary-cli --no-default-features --features i2pcontrol`;
- focused M062 dependency-containment test (`m062_dependency_containment`, 8 tests);
- focused I2PControl authentication tests (`i2pcontrol::auth`, 20 tests);
- retained M061 containment test (`m061_containment`, 7 tests);
- exact changed-path and lockfile review;
- `git diff --check`.

The verification did not require CI/release apparatus and did not broaden into a general dependency cleanup.

## Historical source-completion status

M053/M045 and M046-M048 remain closed. M049/M050/M052 were corrected by M054-M056. M057 is closed planning consistency. M051 remains blocked with its accepted limitation.

## Final status rule

Containment completion does not mean full Proposal 170 support. The containment workstream returned to closed with M061 governing source paths and M062 governing direct dependency ownership. No upstream review or acceptance is implied.
