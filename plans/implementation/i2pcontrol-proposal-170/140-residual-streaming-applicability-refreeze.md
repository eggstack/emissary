# M140 — Residual Streaming Applicability Re-Freeze

Status: **registered / dependency-ready**

Class: invariant / qualification / matrix truthfulness

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`.

Hard dependency:

- M139 closed as complete and remains current runtime/security qualification authority.

Planning baseline:

- repository head at planning start: `7b51da725e83c3ec4a7b806a4e242730b7cb93d7`;
- M095 current matrix: `325 apply / 47 blocked_primitive / 468 not_applicable`;
- M131 closure/map is historical residual authority and remains unchanged;
- M139 closure mechanically requalified the `325/47/468` head.

Pinned reference authority:

- Proposal 170 revision `2026-05-20`;
- Java I2PControl Proposal-170 head `45bb593000408071dd376b78848fdc246dccd964`;
- Java I2P/I2PTunnel snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`;
- exact optional Yosemite revision `59140a2277bf296928d2e8ce39a148182eeff044`.

External sources are read-only. All writes remain internal to `eggstack/emissary`.

## 1. Objective

Re-freeze the applicability of exactly eight residual cells whose M131 blocked disposition depended on generic Proposal/I2PControl setters without a complete family-runtime-consumption proof:

- `Profile:client`;
- `Profile:httpclient`;
- `Profile:ircclient`;
- `Profile:socks`;
- `Profile:socksirc`;
- `Profile:connectclient`;
- `Profile:streamrclient`;
- `ConnectDelay:streamrclient`.

M140 answers one question for each cell: **does the pinned reference tunnel family actually consume the option as an observable runtime behavior, after constructor/family overrides, or is the field affirmatively non-applicable to that family?**

M140 has zero runtime-support promotion budget. It may only retain `blocked_primitive` or reclassify a cell to `not_applicable` with affirmative evidence.

## 2. Why M140 is required

M131 correctly refused to infer non-applicability merely from the absence of a Streamr/TCP analogue. Subsequent direct source review recovered stronger evidence that was not frozen by M131:

- `StreamrConsumer` extends the UDP/datagram `I2PTunnelUDPClientBase`; it does not create an I2P streaming socket whose SYN path could implement `i2p.streaming.connectDelay` or whose streaming connection window could implement Profile.
- `I2PTunnelHTTPClientBase` forcibly sets its connect-delay behavior and removes the `i2p.streaming.maxWindowSize` interactive override; `I2PTunnelConnectClient` inherits this base.
- `I2PTunnelIRCClient` independently removes the max-window override to force bulk behavior.
- `I2PSOCKSTunnel` does the same; `I2PSOCKSIRCTunnel` inherits SOCKS behavior.
- Java's I2PTunnel UI identifies interactive behavior through `i2p.streaming.maxWindowSize == 16`; this must be reconciled with actual constructor overrides and the current Java streaming implementation before any Proposal Profile implementation is planned.

Generic parser/creator acceptance is not enough to prove an applicable runtime option if the constructed tunnel family overwrites or never consumes it.

## 3. Invariants

- Production Rust behavior must not change.
- No Proposal cell may move to `apply`.
- Historical M131/M139 closure files remain immutable.
- M095 remains the machine authority and may be changed only after a cell-complete evidence decision.
- A `not_applicable` decision requires affirmative pinned Proposal/reference family-runtime evidence, not absence of a convenient Emissary primitive.
- A Java I2PControl generic setter cannot outweigh an actual tunnel-family constructor/runtime override without an explicit reference path showing the option survives and is consumed.
- Current unsupported values continue to fail before allocation/effect.
- No Yosemite, Cargo, router, transport, NetDB, crypto, frontend or production-backend change is authorized.
- Full Proposal support remains partial regardless of any count reduction.

## 4. Explicit non-goals

M140 does not:

- implement Profile or ConnectDelay;
- implement `UniqueLocalAddressPerClient` or any other remaining residual;
- reinterpret already closed `Reduce*`, `Close*`, or `NewDest` Streamr semantics;
- edit Java/upstream repositories;
- change a tunnel type/action/API field;
- broaden M061/M062 production allowances.

## 5. Required evidence table

Create `plans/implementation/i2pcontrol-proposal-170/140-residual-streaming-applicability-map.toml` with exactly eight records. Each record must contain:

- canonical option and tunnel family;
- starting M095 disposition;
- Proposal type/applicability evidence;
- Java I2PControl parser/creator behavior;
- Java actual tunnel class and constructor hierarchy;
- exact property emitted, if any;
- exact runtime consumer, if any;
- constructor/runtime override after generic option insertion, if any;
- Yosemite wire capability, separated from runtime semantics;
- current Emissary tunnel data-plane type;
- final disposition;
- rationale and source locations;
- whether a future implementation primitive remains required.

The artifact must record its source SHAs and the exact pre/post M095 counts.

## 6. Cell-specific decision tests

### 6.1 `ConnectDelay:streamrclient`

Trace:

1. Proposal/I2PControl creator acceptance;
2. `StreamrConsumer` construction;
3. `I2PTunnelUDPClientBase` session creation;
4. whether any streaming `ConnectionOptions` / SYN-delay code is entered;
5. whether the value affects UDP/datagram session start, control pinger, send or receive behavior.

Reclassify to `not_applicable` only if the pinned runtime positively proves there is no reference ConnectDelay event/consumer for Streamr despite generic setter reachability.

Do not invent a datagram delay timer merely to preserve a blocked cell.

### 6.2 `Profile` family-by-family

For each of seven client families, trace the final effective `i2p.streaming.maxWindowSize` / profile configuration **after constructor overrides** and identify the actual streaming manager/connection-options consumer.

Required family checks:

- plain `client`: prove whether caller-provided interactive/max-window configuration survives and is consumed;
- `httpclient`: inspect `I2PTunnelHTTPClientBase` forced bulk behavior;
- `connectclient`: prove inheritance/override behavior from HTTP base;
- `ircclient`: inspect its explicit bulk override;
- `socks`: inspect its explicit bulk override;
- `socksirc`: prove inheritance from SOCKS;
- `streamrclient`: prove UDP/datagram vs streaming ownership.

If contemporary reference code treats the legacy Profile property itself as ignored but still implements the I2PTunnel Profile field through `maxWindowSize`, record that distinction. The Proposal cell disposition follows observable tunnel-family semantics, not property-name folklore.

## 7. Expected but non-binding hypothesis

The planning research suggests the likely result is:

- retain `Profile:client` as `blocked_primitive`;
- reclassify `Profile` for HTTP, IRC, SOCKS, SOCKS-IRC, CONNECT and Streamr to `not_applicable`;
- reclassify `ConnectDelay:streamrclient` to `not_applicable`.

That would produce `325 apply / 40 blocked_primitive / 475 not_applicable`.

**This count is a hypothesis, not an acceptance criterion.** If pinned runtime evidence contradicts any row, retain the blocker and mechanically compute the correct count. Do not force `325/40/475`.

## 8. Required changes and path budget

Production paths: **none**.

Authorized planning/test/doc paths:

- new `140-residual-streaming-applicability-map.toml`;
- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml` only for evidence-backed disposition/count changes;
- `plans/implementation/i2pcontrol-proposal-170/105-residual-option-audit.toml` only if it is active current-head accounting rather than immutable historical evidence;
- `emissary-cli/tests/m095_full_support_matrix.rs`;
- `emissary-cli/tests/m105_residual_option_audit.rs` only if needed to reflect current authority;
- a focused M140 matrix/applicability guard if the existing guards cannot encode source-backed row expectations cleanly;
- `plans/registry.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`;
- active user-facing Proposal support docs only where counts/residual descriptions are current-head claims;
- `plans/closure/i2pcontrol-proposal-170/140-closure.md` at closure.

Any required production Rust/Cargo/Yosemite change is a stop condition.

## 9. Work packages

### WP1 — Freeze baseline mechanically

- record exact HEAD/worktree state;
- parse M095 and prove `325/47/468`, 840 total;
- list the exact 47 blockers and confirm all eight M140 candidates are present;
- snapshot active docs that state residual counts.

### WP2 — Pin reference source excerpts

For each candidate, record source file, class/function/property and pinned SHA. Prefer actual runtime class/consumer over UI wording or generic creator code.

### WP3 — Produce cell-complete applicability map

Fill all eight records and independently review every proposed N/A row against the M131 `not_applicable` standard.

### WP4 — Reconcile machine authority

Only after WP3 is complete:

- update M095 rows/counts if required;
- update current residual guards/docs;
- leave historical closure evidence unchanged;
- state the exact retained Profile cell set that M143 may target.

### WP5 — Verify containment and closure readiness

Run matrix/residual/containment suites and prove `git diff` contains no production source.

## 10. Failure, cancellation, restart, contention

M140 changes no runtime state and therefore introduces no new runtime failure/cancellation surface. Its correctness failure mode is evidence misclassification.

To guard against that:

- every disposition is independently source-cited;
- every proposed N/A row includes the actual family constructor/runtime path;
- count changes are mechanically derived from row changes;
- no prose-only count edit is accepted.

## 11. Verification

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit --test m061_containment --test m062_dependency_containment --no-fail-fast
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo fmt --all -- --check
git diff --check
```

Also mechanically verify:

- exactly eight M140 evidence records;
- every changed M095 disposition is among those eight;
- `apply` count is unchanged from 325;
- no production path changed.

## 12. Acceptance criteria

M140 closes only when:

1. all eight candidate cells have pinned actual-runtime applicability evidence;
2. no cell is promoted to `apply`;
3. every N/A change meets the affirmative-evidence rule;
4. M095, current-head tests and active docs mechanically agree on final counts;
5. the retained Profile implementation set is explicit;
6. production behavior and unsupported fail-before-allocation behavior are unchanged;
7. M061/M062 containment remains at least as strict;
8. M141 is identified as the next successor only after closure.

## 13. Stop conditions

Stop and close M140 as blocked/corrective-required if:

- reference snapshots cannot establish actual family runtime consumption;
- a candidate's behavior differs by ambiguous reference path that cannot be resolved without executing or inspecting an unavailable source;
- a disposition change would require production behavior to stay truthful;
- M095 cannot be reconciled without changing cells outside the eight-candidate set.

## 14. Closure evidence required

Record:

- implementation/closure HEADs;
- eight-row evidence-map hash;
- exact source SHAs/locations;
- pre/post matrix row diff and counts;
- proof of zero `apply` promotions;
- production-path diff proving no runtime changes;
- verification outcomes;
- historical-record immutability review;
- exact M141 readiness decision;
- attestation that all external access was read-only and no upstream contribution artifact/contact was made.