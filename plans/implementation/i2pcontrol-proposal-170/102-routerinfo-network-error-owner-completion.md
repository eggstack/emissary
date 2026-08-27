# M102 — RouterInfo Canonical Network-Error Owner Completion

Status: ready; dependency M095 closed

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Canonical requirements:

- `plans/000-long-term-specification.md`;
- `plans/003-planning-process.md`;
- ADR-0004 full-support/minimal-core boundary;
- M050 network status/testing source work;
- M055 network-error truthfulness corrective;
- M056 current RouterInfo 37/1/5 reclosure;
- M061 exact source-containment authority.

Planning baseline: `630a8fd1cd4e5943fcde0b5c16f5fc1e88b5d207` plus accepted M095 owner/path audit when dependency-ready.

Pinned external contract:

- `i2p.router.net.error` — IPv4 network error code, `int`;
- `i2p.router.net.error.v6` — IPv6 network error code, `int`;
- Proposal 170 revision `2026-05-20`, adopted from i2pd.

Classification: capability / containment / neutral inspection infrastructure.

## 1. Objective

Make the two Proposal 170 network-error rows truthfully available from explicit canonical IPv4/IPv6 error-reason state while adding the smallest possible neutral lower-layer observation surface and leaving all Proposal 170/i2pd wire-code mapping in `emissary-cli/src/i2pcontrol/**`.

M055 correctly removed the prior `0 / No error` fallback because Emissary had no canonical error-reason owner. M102 does not restore that fallback. It creates or exposes explicit state only at the runtime owners that already determine/observe the relevant network condition.

## 2. Hard readiness gate

M102 MUST NOT execute until M095 closes with:

- the exact i2pd/Proposal 170 error code semantic table;
- an audit of all current IPv4/IPv6 reachability/testing/transport owners and writers;
- proof that the required reason cannot be derived truthfully from existing accepted inspection state alone;
- an exact, minimal list of non-I2PControl production files that must change;
- a field-by-field writer table naming the event that sets/clears each neutral error reason;
- confirmation that no proposed writer would alter transport/reachability decisions.

If M095 finds an already-existing canonical explicit error source, update M102 to consume that source and remove unnecessary core-change authorization before registration.

## 3. Neutral domain model

Core/runtime code must not know Proposal 170 integer codes or names.

Preferred neutral concept:

```text
NetworkFamily = V4 | V6
NetworkErrorReason = implementation-neutral finite enum
NetworkErrorSnapshot {
    v4: Option<NetworkErrorReason>,
    v6: Option<NetworkErrorReason>,
    generation/timestamp as needed for truthfulness
}
```

Exact enum variants come from the runtime conditions that map one-to-one onto the pinned error meanings. Do not add variants merely to mirror integer slots that Emissary can never observe.

I2PControl owns the total mapping:

```text
Option<NetworkErrorReason> + explicit source validity
    -> Proposal 170/i2pd integer result or truthful unavailable/error state
```

`None` must not automatically serialize as code `0` unless `None` has been explicitly defined by the canonical owner to mean `No error` after a completed status evaluation.

## 4. Writer/source rules

Every neutral error state transition requires an existing authoritative event, for example a transport bind/address-family failure, explicit firewall/reachability test result, peer-test outcome, or other M095-verified condition.

Rules:

- one canonical owner per family/reason;
- state is observational output of existing behavior, not an input to transport decisions;
- recording a reason cannot change retries, peer selection, address advertisement, tunnel building, or reachability testing;
- clearing a reason occurs only on an explicit successful/changed condition defined by the source table, not on API read;
- stale conditions must have a defined generation/lifetime relation to current reachability/testing state;
- request frequency has no effect.

Do not add network probes solely to populate the field.

## 5. Preferred path budget

M095 must narrow this before M102 becomes ready. Candidate paths are restricted to paths already present in the accepted M061 containment manifest and should be minimized from this set:

Core neutral observation/owner candidates:

- `emissary-core/src/events.rs`;
- `emissary-core/src/inspection.rs`;
- `emissary-core/src/transport/mod.rs`;
- only the exact NTCP2/SSU2 owner module(s) that M095 proves originate the adopted reason.

I2PControl consumers:

- `emissary-cli/src/i2pcontrol/observers.rs` and/or `observability.rs`;
- `emissary-cli/src/i2pcontrol/router_info.rs`;
- `emissary-cli/src/i2pcontrol/router_info_handler.rs`;
- `emissary-cli/src/i2pcontrol/production.rs` only if the neutral handle must be composed there;
- focused tests/docs/M095 matrix updates.

The following are not authorized merely because they exist in M061's historical allowlist:

- crypto modules;
- NetDB mutation modules;
- tunnel builder/pool algorithms;
- SAM/Yosemite application runtime;
- router context behavior unrelated to inspection composition;
- broad transport session refactors.

M102 must update the exact containment manifest/guard only if required by its preauthorized changed paths. It must not broaden prefixes/globs.

## 6. Invariants

1. Wire codes exist only in I2PControl.
2. Core enum/state names are Proposal-170-agnostic.
3. State is passive observation; no router decision consumes it.
4. Every state writer corresponds to an existing real runtime condition.
5. No API read mutates/clears state.
6. `No error` is emitted only from explicit valid state, never source absence.
7. v4 and v6 are independently owned/updated.
8. Collections/tasks remain unchanged or bounded; no new polling daemon/probe.
9. No secret/socket/mutable transport handle crosses inspection.
10. No upstream interaction occurs.

## 7. Explicit non-goals

M102 MUST NOT:

- change reachability/firewall algorithms;
- add new active probes;
- change SSU2 peer-test behavior;
- change transport retry/backoff/address selection;
- add routing/tunnel/NetDB behavior;
- add Proposal 170 terms/types to core;
- use status/testing adjacency as a substitute for explicit error reason unless M095 proves semantic identity;
- implement banned peers/news/transit metrics;
- widen M061 with broad globs;
- add dependencies/CI/release machinery;
- contact upstream.

## 8. Ordered work packages

### A. Freeze code mapping and writer table

From M095/reference evidence, document each adopted integer code, neutral internal reason, authoritative runtime event, clear event, family, and source path.

If any code needed for full support has no real Emissary condition, determine whether it is simply unreachable/not-applicable in this router or whether the contract requires representability. Do not fabricate a writer.

### B. Add minimal neutral owner state

Prefer extending an existing bounded event/inspection state rather than creating a new subsystem/task. Keep synchronization simple and non-blocking.

### C. Wire exact runtime observations

At the minimum existing owner sites, record/clear the neutral reason as a side observation after/beside the existing decision. The observation write must not determine the decision.

### D. Expose bounded snapshot

Use the accepted inspection pattern: copied/finite enum state only, no mutable handles/sockets/channels/private material.

### E. Map in I2PControl

Implement exact integer mapping and availability semantics. Keep current direct-presence request behavior and types unchanged.

### F. Add regression/containment guards

Prove absence of `Proposal 170`, JSON-RPC, wire integer mapping, or I2PControl policy in changed core declarations. Prove no new unauthorized core path.

## 9. Failure/restart/contention semantics

- observation write failure/panic must not be able to change transport behavior; choose infallible/bounded state updates where practical;
- state begins `unknown/not-yet-evaluated`, not `No error`, until the authoritative owner reaches the defined evaluation point;
- restart resets transient network-error state and rebuilds from current runtime observations; no persistence required;
- concurrent transport observations use deterministic last-current-generation state according to M095 source semantics;
- no lock held across network I/O or await;
- RouterInfo snapshot failure returns the existing truthful error/unavailable behavior rather than zero.

## 10. Compatibility/migration

No public schema change and no persistent migration. Existing clients simply gain truthful integer results for the two requested keys after valid state exists.

Default/no-I2PControl router execution may still record neutral error state only if the state belongs naturally to an existing owner and has negligible bounded cost. If the state exists solely for I2PControl, prefer feature-gated passive observation without changing transport behavior. M095 must choose the smallest valid form.

## 11. Tests

At minimum:

- every neutral reason -> exact wire code fixture;
- explicit successful evaluation -> code 0 only where semantically valid;
- source-uninitialized does not serialize as No error;
- independent v4/v6 state;
- writer event and clear event tests at each changed owner;
- API reads do not mutate state;
- restart/reset behavior;
- no transport behavior difference with observation enabled/disabled where testable;
- changed-core static scan for Proposal170/JSON-RPC/wire-code terms;
- M061 exact path containment;
- current status/testing rows remain unchanged.

## 12. Verification

Because M102 is the one planned neutral-core milestone, run both focused core and I2PControl tests:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-core
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m063_feature_reachability
git diff --check
```

Run clippy for changed packages if it is part of the repository's accepted local verification. Do not introduce formatter-only churn across unrelated core files because of nightly/stable rustfmt drift.

## 13. Documentation/static guards

Update M095's two rows only after explicit source evidence. Update the source/truthfulness roadmap/support docs to say the previous M055 demotion remains historically correct but is superseded for current production by M102 if closure succeeds.

Static guards must make the core/I2PControl asymmetry durable: neutral reasons in core, integer/wire mapping in I2PControl.

## 14. Acceptance and stop conditions

M102 closes only if:

- M095 exact owner/path audit is satisfied;
- both v4/v6 rows come from explicit valid state;
- no fabricated `0` path remains;
- changed core paths are exact/minimal and already justified by M061 or explicitly amended;
- core observation cannot affect router decisions;
- core contains no Proposal 170/wire policy;
- focused/broad tests pass with no high/medium finding;
- no upstream interaction occurred.

Stop and leave full-support completion blocked if:

- the required reasons cannot be observed without new active network probes;
- implementation would change transport/reachability algorithms;
- exact owner paths cannot be bounded before code;
- satisfying the field would require a broad new core subsystem.

## 15. Closure evidence required

Create `plans/closure/i2pcontrol-proposal-170/102-closure.md` with:

- M095 owner/code/path matrix;
- exact implementation heads and changed paths;
- neutral reason domain and writer/clear table;
- proof no Proposal 170/wire mapping entered core;
- v4/v6 source/wire-code tests;
- router-behavior non-interference review;
- core/I2PControl/containment verification results;
- updated RouterInfo matrix totals;
- unresolved findings/severity;
- internal-only/no-upstream attestation.

## 16. Internal-only rule

All writes remain internal to `eggstack/emissary`. External i2pd/I2P/reference code is read-only evidence. No upstream issue/PR/review/submission/merge/contribution activity is authorized.
