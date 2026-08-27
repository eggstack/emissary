# Emissary Planning System

This directory separates durable architectural direction from temporary implementation handoffs.

## Canonical documents

The following files define the authoritative direction for the active planning scope:

- `000-long-term-specification.md` — normative end-state requirements and invariants.
- `001-terminology-and-domain-model.md` — normative language and ownership model.
- `002-long-term-roadmap.md` — dependency-ordered capability roadmap.
- `003-planning-process.md` — planning, handoff, closure, and archive rules.

Ordinary implementation work MUST reference these documents rather than silently weakening or rewriting them. Changes to canonical direction require an explicit architecture decision or direct maintainer instruction.

## Planning hierarchy

```text
Canonical specification and terminology
        |
        v
Architecture decision records
        |
        v
Long-term roadmap
        |
        v
Subsystem roadmaps
        |
        v
Milestone implementation plans
        |
        v
Implementation and verification
        |
        v
Closure records and archive
```

## Directory roles

- `adrs/` — durable architecture decisions. Accepted decisions are superseded, not rewritten.
- `subsystems/` — coherent, dependency-ordered workstream roadmaps.
- `implementation/` — bounded milestone plans handed to implementation agents.
- `closure/` — evidence-based completion records.
- `archive/` — completed or superseded interim planning retained for traceability.
- `registry.md` — compact index of active roadmaps, executable plans, blockers, and closure state.

## Core rule

Canonical documents state what Emissary must become and what must remain true. Interim plans state what an agent should implement next against a specific repository baseline.

Implementation agents MUST NOT add commit-specific mechanics, transient test counts, or corrective checklists to canonical documents.

## Planning lifecycle

1. Identify applicable canonical requirements and invariants.
2. Record any unresolved durable decision in `adrs/`.
3. Create or update the relevant subsystem roadmap.
4. Select one dependency-ready milestone.
5. Write a bounded plan under `implementation/`.
6. Register the plan in `registry.md`.
7. Implement and verify the milestone.
8. Write a closure record under `closure/`.
9. Update the registry and subsystem status.
10. Archive completed or superseded interim documents when they are no longer active.

No milestone is complete merely because code landed. Completion requires the evidence defined by its implementation plan and subsystem roadmap.

## Work classification

Every roadmap and implementation plan MUST distinguish:

- **Invariant** — a property that must always remain true.
- **Capability** — externally observable behavior.
- **Infrastructure** — internal machinery required by capabilities.
- **Polish** — diagnostics, ergonomics, cleanup, performance tuning, or documentation.

Infrastructure and polish MUST NOT be represented as completed capability without end-to-end acceptance evidence.

## Naming conventions

- ADR: `adrs/ADR-NNNN-short-title.md`
- Subsystem roadmap: `subsystems/<subsystem>-roadmap.md`
- Implementation plan: `implementation/<subsystem>/NNN-short-title.md`
- Closure record: `closure/<subsystem>/NNN-status.md`
- Archived document: retain its original relative structure under `archive/`

Use stable subsystem names. Do not encode dates in filenames unless the document is inherently time-bound.

## Current Proposal 170 workstream

The active workstream is the exact internal implementation of I2P Proposal 170 through I2PControl, pinned to the proposal revision created/updated `2026-05-20`.

The workstream has progressed through three architectural phases:

1. contract completion and truthful partial runtime under ADR-0001/ADR-0002;
2. all-twelve-tunnel runtime completion and application-layer security filtering under ADR-0003;
3. full-support completion under ADR-0004, covering the remaining RouterInfo sources, operational AddressBook SetConfig semantics, all applicable TunnelManager option semantics, and final live interoperability/reclosure.

The preferred production boundary remains `emissary-cli/src/i2pcontrol/**`. Lower-layer changes are exceptional and require an explicitly registered plan naming the canonical owner and exact path budget before implementation.

The active phase is defined by:

- `adrs/ADR-0004-pinned-full-proposal-170-completion-boundary.md`;
- `subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`;
- `registry.md`.

Only the dependency-ready plan registered in `registry.md` is executable. Prewritten future plans are continuity documents, not authorization to skip dependencies.

This workstream remains internal-only. External I2P/I2P+/i2pd/Java I2P/Yosemite repositories and specifications are read-only evidence. No upstream issue, pull request, review, merge, submission, contribution preparation, adoption request, or maintainer contact is authorized.