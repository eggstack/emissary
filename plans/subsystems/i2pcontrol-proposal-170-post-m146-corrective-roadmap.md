# I2PControl Proposal 170 Post-M146 Corrective Roadmap

Status: **historical through M154 / superseded for current execution**

This roadmap governed the corrective sequence after M146 through M153/M154. It is retained as historical planning context, but it is no longer the execution authority after M154 disposition C showed that the general configurable Destination `SigType` path could not safely gate the LeaseSet-security tail.

Current execution authority:

- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

## Historical outcomes governed here

- M146 closed blocked with 4 `UseOutproxyPlugin` cells remaining unsupported.
- M153 closed complete as current whole-surface runtime/security qualification authority at `336/29/475`.
- M154 closed complete with disposition C after freezing the destination-capable SigType domain and proving the general M147 primitive could not truthfully satisfy it under current security/dependency policy.
- M147 path is therefore closed blocked; M148 remains blocked behind it.

## Why execution moved to a new roadmap

Post-M154 reference research established two facts not represented in this roadmap's original dependency chain:

1. modern Encrypted LeaseSet2 can start from an existing type-7 Ed25519 Destination and derive a narrower type-11 Red25519 blinded signing key without implementing user-selectable/persistent Destination `SigType`;
2. `EncryptLeaseSet`, `OptionalLookup`, and `LeaseSetClientAuths` are coupled through the Proposal's ten-value encryption mode domain, so the old M149 -> M150 -> M151 sequencing is not a truthful capability-promotion order.

The current roadmap therefore decomposes the LeaseSet tail into M155-M162 and re-gates M152 on that line.

## Historical authority preserved

Historical closure records for M146, M153 and M154 remain immutable and authoritative for what those milestones actually proved. This file does not reopen or alter their dispositions.

M149-M151 remain in-tree only as superseded historical drafts. M147/M148 remain blocked. M146 remains blocked unless a separate future architecture/security plan supersedes it.

All containment/security rules from M061/M062/M093 and the external read-only boundary remain in force.