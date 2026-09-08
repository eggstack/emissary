# M143 — Streaming Profile Runtime Completion

Status: **deferred / unregistered; hard-depends on M142 and M140 closure**

Class: infrastructure + capability / streaming runtime semantics

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`.

Target cells:

- exactly the `Profile` cells that M140 leaves `blocked_primitive`.

M143 MUST be amended before registration to name the exact retained family set and the M140-closure matrix baseline. It must not assume the planning hypothesis that only plain `client` survives.

## 1. Objective

Implement Proposal `Profile` only where the pinned reference tunnel family actually consumes an interactive/bulk streaming configuration, and only by connecting the I2PControl option to a real neutral Emissary streaming configuration owner.

Serializer reachability, generic SAM additional options, or persistence do not count. The selected Profile must measurably change the effective streaming connection/window behavior owned by Emissary.

## 2. Hard dependencies and readiness

Hard dependencies:

- M140 closed with a cell-complete Profile applicability verdict;
- M141 and M142 closed so I2PControl-local residuals ahead of this core seam are complete and the current matrix is reconciled.

Before registration, inspect the then-current streaming implementation and freeze exact production owners. Registration is blocked until the plan names the smallest exact-file core path budget for:

- the neutral session/stream configuration input;
- the actual streaming connection/window consumer;
- any SAM session handoff required to carry that neutral value.

If the active streaming implementation still cannot expose a bounded per-session/per-stream max-window/profile setting without broad redesign, stop and leave retained Profile cells blocked.

## 3. Pinned semantic contract

M140 closure supplies the family applicability set. M143 separately freezes the value semantics from the pinned Java streaming/I2PTunnel reference:

- exact accepted Proposal value(s), including the `interactive` spelling/domain;
- the effective reference mapping to streaming configuration, including `i2p.streaming.maxWindowSize` or its current equivalent;
- reference default when Profile is omitted;
- interaction with family constructor overrides;
- whether the setting is session-manager default, per-socket override, or both;
- whether an explicit disabled/bulk value exists and how it is represented;
- bounds and failure behavior for unsupported values.

Do not encode a legacy property name into core if the durable lower-layer concept is simply a bounded streaming window/profile configuration.

## 4. Ownership and initial path budget

Proposal parsing/validation/mapping remains under I2PControl, expected paths:

- `emissary-cli/src/i2pcontrol/backends/options.rs`;
- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs`;
- only the backend files for M140-retained families;
- `emissary-cli/src/i2pcontrol/domain/tunnel.rs` / `tunnel_manager.rs` only if exact typed extraction is required.

Potential neutral core owners must be re-inspected before registration. Candidate areas are limited to the existing SAM streaming/session implementation under:

- `emissary-core/src/sam/session.rs`;
- exact files under `emissary-core/src/sam/protocol/streaming/` that own connection configuration/window enforcement.

This candidate list is **not production authorization**. Before M143 is registered, replace it with exact files and amend M061/M062. No broad `sam/**` allowance is permitted.

No NetDB, crypto, transport, tunnel-pool, Cargo/dependency, Yosemite, frontend, startup-tunnel or `.github/**` change is expected.

## 5. Neutral lower-layer contract

The core seam must be Proposal-free and reusable by any SAM consumer. It should express only the actual streaming primitive, for example a bounded connection profile/max-window configuration.

Required properties:

- initialized before the affected stream manager/connection becomes active;
- immutable for an active stream unless the reference supports live mutation;
- bounded to protocol-safe values;
- no global/router-wide mutable setting;
- shared sessions have one compatible effective profile or reject incompatible sharing before allocation;
- no per-packet logging or unbounded tracking;
- omitted option preserves current upstream-compatible default.

If the reference Profile distinction is not operational in the pinned/current streaming implementation, do not invent one. M143 must remain blocked or M140 must be corrected through a new applicability plan.

## 6. I2PControl validation and shared-session compatibility

For each retained family:

- validate the exact Proposal type/value before listener/session allocation;
- map only validated values to the neutral streaming configuration;
- include the effective Profile in shared-session compatibility so definitions with incompatible profiles cannot silently share one streaming manager;
- ensure edits that require a profile change follow existing restart/transaction semantics and do not mutate a live shared session behind another definition;
- preserve exact Get/round-trip behavior without adding aliases.

Families M140 marked `not_applicable` must continue to reject/omit the option according to matrix authority and must never inherit the new core seam accidentally.

## 7. Failure, cancellation, restart and contention

- Profile validation is synchronous/pre-allocation.
- Session construction failure leaves no listener or committed replacement generation.
- Cancellation during session setup cannot publish a partially configured stream manager.
- Restart creates a new generation with the new effective profile; stale streams retain only their old generation until teardown.
- No lock is held across SAM connection, destination resolution or stream I/O.
- Shared-session admission/rejection is deterministic under concurrent starts.

## 8. Work packages

### WP1 — post-M140 semantic/path freeze

Amend this plan with the exact retained cells, accepted value domain, reference mapping and exact core files.

### WP2 — neutral streaming configuration primitive

Implement the smallest reusable lower-layer configuration field and consume it at the real streaming window/connection owner. Add core-only tests proving the configuration changes effective runtime state/behavior.

### WP3 — I2PControl mapping

Parse/validate Profile, map it into the neutral configuration, and update shared-session compatibility and backend gates for only retained families.

### WP4 — observable behavior tests

Use deterministic streaming fixtures to prove interactive/default settings produce the reference-distinct effective window/profile behavior. Do not rely solely on serialized SAM command strings.

### WP5 — matrix/containment/closure

Promote only proven retained cells, update M061/M062 exact paths, M095/M105/docs/registry and closure evidence.

## 9. Focused tests

Required after the exact retained set is known:

1. omitted Profile preserves current default behavior;
2. exact accepted Profile value produces the pinned effective window/profile;
3. invalid type/value fails before allocation and is not echoed in errors;
4. effective configuration reaches the actual stream connection/window owner;
5. an observable deterministic test distinguishes default vs interactive behavior/state;
6. shared sessions with equal Profile remain compatible;
7. incompatible Profile values cannot silently share;
8. edit/restart applies the new profile only to the successor generation;
9. cancellation cannot leave a partially configured manager;
10. every M140-N/A family remains unaffected/rejected and has no accidental core inheritance;
11. feature-disabled/default builds remain unchanged.

## 10. Matrix promotion budget

Maximum promotion budget equals the exact number of Profile cells retained as `blocked_primitive` by M140. This plan must be amended before registration to replace this sentence with the exact number and family list.

No family may be promoted merely because a generic field appears on the SAM wire.

## 11. Broad verification

```text
cargo check -p emissary-core
cargo test -p emissary-core --no-fail-fast
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

If an amended exact core path participates in no-std, add the relevant no-std check before registration.

## 12. Acceptance criteria

M143 closes as complete only when:

- M140-retained cells and no others are implemented;
- a neutral core streaming primitive is consumed by the actual runtime owner;
- effective behavior matches pinned Profile semantics and is deterministically observable;
- default behavior is unchanged when omitted;
- shared-session compatibility is exact;
- core contains no Proposal/I2PControl terminology or policy;
- actual non-policy changed paths are exact-file authorized in M061/M062;
- matrix/docs/registry match runtime evidence;
- no medium/high streaming correctness or containment defect remains.

## 13. Stop conditions

Stop and leave cells blocked if:

- M140 leaves no applicable Profile cells (in that case M143 becomes unnecessary and should be closed/superseded without production work);
- exact Profile behavior is ignored by the pinned runtime and cannot be distinguished from default;
- implementation would require a router-global streaming redesign or broad unrelated core churn;
- only Yosemite wire serialization can be achieved;
- family overrides would make the setting inert.

## 14. Closure evidence required

Record the amended exact target set/path budget, pinned semantic table, core primitive tests, observable profile behavior, shared-session compatibility evidence, changed paths, matrix delta, containment updates, broad verification, implementation SHA, unresolved findings and M144 readiness. External reference access remains read-only.