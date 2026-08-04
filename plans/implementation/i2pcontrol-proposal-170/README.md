# Proposal 170 Implementation Handoffs

Status: corrective pass required

This directory contains bounded internal implementation and closure handoffs for the I2PControl Proposal 170 subsystem.

Authoritative direction:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`
- `plans/closure/i2pcontrol-proposal-170/029-closure-invalidation.md`

Pinned external authority:

- Proposal 170 `I2PControl Expansion`, Open, created/updated `2026-05-20`
- existing I2PControl authentication and JSON-RPC contract

## Internal-only rule

These handoffs are internal to `eggstack/emissary`.

No plan authorizes:

- an upstream issue, pull request, merge request, discussion, review request, or patch submission;
- upstream review, feedback, approval, adoption, or merge solicitation;
- pushing branches, commits, tags, patches, artifacts, or releases to an upstream remote;
- upstream maintainer outreach;
- preparation of an upstream contribution package, patch series, submission checklist, or merge plan;
- connector/API writes against any upstream or third-party repository.

External specifications and reference implementations may be inspected read-only for internal correctness. All writes must remain in `eggstack/emissary` unless a future explicit maintainer directive supersedes the normative planning policy.

Violation is a stop condition and invalidates affected evidence.

## Current handoff

| Handoff | Status | Plan | Dependency |
|---|---|---|---|
| M030 — AddressBook destination and owner coherence | ready | `030-addressbook-destination-owner-coherence.md` | none |

No successor reclosure plan is registered. It may be created and registered only after M030 has a frozen implementation/test head and accepted implementation closure.

## Current defect boundary

M029 is historical invalidated evidence. The active corrective slice is limited to:

- active Base64 lookup bypassing the control owner through a stale legacy destination file;
- first activation seeding published Proposal 170 entries from Base32 cache values instead of full destinations;
- active download merge retaining an incomplete seed;
- missing update/delete/full-destination regressions.

Authoritative invalidation:

- `plans/closure/i2pcontrol-proposal-170/029-closure-invalidation.md`

## Scope rule

Primary production work should remain in:

- `emissary-cli/src/i2pcontrol/production.rs`;
- directly affected `emissary-cli/src/i2pcontrol/**` adapters and tests.

Changes outside the I2PControl crate are permitted only where the shared runtime lookup owner must change:

- `emissary-cli/src/address_book.rs` for owner-aware Base64 lookup, bounded full-destination loading/validation, one purpose-specific import/repair seam, and focused tests;
- `emissary-cli/src/main.rs` only for one narrow activation input or call if required.

M030 must not modify `emissary-core/**`, implement missing tunnel data planes, add RouterInfo sources, redesign resolver policy, add a second AddressBook authority, introduce bidirectional synchronization/provenance/tombstones, add dependencies, expand CI/release machinery, or prepare upstream work.

## Target behavior

### Disabled/default mode

Retain M028 exactly:

- legacy files drive lookup and download persistence;
- control state is ignored and untouched;
- no mutation handle exists.

### First enabled activation

- import bounded, validated full destination files;
- publish one complete control generation before service startup;
- derive Base32 indexes from full destinations;
- never use the Base32 cache as a Proposal 170 destination source.

### Existing enabled authority

- retain valid full destinations;
- repair historical Base32-seeded published values only from matching validated full destination files;
- fail I2PControl activation on unrepairable invalid values without changing prior files;
- do not silently merge arbitrary disabled-period edits into established control state.

### Active lookup and downloads

- active owner is authoritative for Base32 and Base64 lookup;
- update/delete cannot fall through to stale legacy files;
- active downloads store validated full destinations and cannot preserve incomplete seeds.

## Retained history

| Milestone | Retained result |
|---|---|
| M020 | base I2PControl authentication/token/error and JSON-RPC correctness |
| M021 | exact TunnelManager wire, validation, atomic persistence, secret boundary |
| M022 | enabled-mode runtime AddressBook authority; destination/lookup coherence reopened by M030 |
| M023 | startup tunnel inventory and ClientServicesInfo lifecycle/address truthfulness |
| M024 | recoverable bounded SAM observation |
| M025 | exact 43-selector RouterInfo contract/source matrix |
| M026 | no feasible additional bounded authoritative RouterInfo sources |
| M027 | literal conformance evidence; final disposition historically invalidated |
| M028 | compile-time/runtime AddressBook feature isolation and optional dependency ownership |
| M029 | independent review evidence; final disposition invalidated |

Current source matrix remains:

- 16 available;
- 1 protocol-permitted neutral;
- 26 unavailable.

Missing tunnel types remain explicit unsupported runtimes under ADR-0001.

## M030 handoff rule

M030 owns one bounded corrective objective:

- make the active owner authoritative for normal Base32 and Base64 resolution;
- import and expose structurally valid full destinations;
- repair bounded historical Base32-seeded published values or fail closed;
- retain M028 feature isolation and transition semantics;
- add regressions spanning API, RouterInfo, Base32, Base64, restart, and stale-file cases;
- keep production changes outside `i2pcontrol/**` minimal and explicitly justified;
- produce an implementation disposition and frozen head.

M030 must stop rather than introduce a new persistence schema, cross-store transaction framework, provenance/tombstone model, core change, general resolver redesign, or unrelated method-family work.

## Handoff discipline

The M030 implementation disposition must contain:

- implementation commits and frozen head;
- exact changed files;
- justification for every production file outside `emissary-cli/src/i2pcontrol/**`;
- before/after failing regression evidence;
- requirement-to-evidence matrix;
- focused and broad command outcomes;
- failure/restart/cancellation/contention evidence;
- compatibility and migration effects;
- security review;
- unresolved findings with severity;
- scope/no-upstream attestation.

A successful commit or broad test count is not closure by itself.

## Verification rule

Required bounded matrix:

```bash
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features address_book
cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings

cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol address_book
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m027_literal_fixtures
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

Use targeted formatting and `git diff --check`.

M030 does not authorize repair of unrelated `emissary-core` test debt. Remote CI, platform matrices, release checks, coverage gates, fuzz campaigns, network farms, soak tests, submission checks, and generated evidence bundles are not required.

## Final-status rule

The current status is `corrective pass required`.

After M030 closes, a distinct final-head review may select:

- `partial Proposal 170 support` when every implemented/claimed dimension is exact and evidenced but sources/runtimes remain unavailable;
- `corrective pass required` for any unresolved high/medium defect;
- `blocked` when required evidence cannot be obtained or the external contract changed.

`closed internally against pinned revision` is not expected under the current scope because 26 RouterInfo sources and missing tunnel data planes remain unavailable/unsupported.

No final status implies upstream review, acceptance, certification, adoption, or merge.