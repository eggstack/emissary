# M088 — Pre-Accept Server Admission Boundary Corrective

Status: closed — Tier 3 unsupported lower-layer semantic; see M088 closure

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Corrective predecessors / controlling evidence:

- M074: `plans/implementation/i2pcontrol-proposal-170/074-server-admission-and-rate-limit-hardening.md`;
- M083: `plans/implementation/i2pcontrol-proposal-170/083-admission-capacity-and-trusted-destination-exactness-corrective.md`;
- M085 closure: `plans/closure/i2pcontrol-proposal-170/085-closure.md`;
- M087 closure when available.

Planning baseline: `2b01bfd11ebcd768fcd5488f18b063ac336931a2`.

Classification: corrective pass / lower-layer admission feasibility and bounded hardening.

## 1. Objective

Determine whether the accepted-stream server family can enforce meaningful connection pressure limits **before application-visible `session.accept()` completes**, and implement the narrowest supported lower-layer bound if it can be done without contaminating the wider Emissary router/core architecture.

Current shared accepted-server logic creates a persistent Yosemite stream session and then performs:

```text
session.accept()
  -> authenticate/canonicalize peer Destination
  -> ServerAdmissionState::try_acquire(...)
  -> spawn bounded handler
```

M074/M083 correctly bound application-visible handler concurrency and per-peer/aggregate admission state, but the admission decision occurs only after Yosemite/SAM has completed enough stream setup to return an accepted stream. When the application-level global limit is saturated, repeated inbound attempts can therefore continue to impose lower-layer stream/session work even though they are immediately rejected above the handler boundary.

The security goal is to reduce this work-amplification window if the existing SAM/I2P streaming boundary already supports it or can expose it through a very narrow fork-local interface.

This milestone is explicitly **feasibility-gated**. It must not manufacture a second SAM stack, fork a broad dependency surface, or modify router internals merely to claim a pre-accept limit exists.

## 2. Threat model and severity

The concern is resource exhaustion and active timing/load correlation, not direct identity disclosure.

An attacker can create many I2P Destinations, so per-Destination limits are not Sybil resistance. Their value is bounded fairness and amplification control. The existing application admission remains necessary even if a lower-layer limit is added.

A lower-layer control is useful only if it actually constrains work before or during stream establishment. A renamed post-`accept()` counter does not satisfy M088.

## 3. Reference behavior to investigate

Use external implementations and specifications as read-only evidence.

At minimum compare:

- Java I2P streaming/session controls including `i2p.streaming.maxConcurrentStreams`;
- Java I2P per-peer and aggregate connection-rate controls such as `maxConnsPerMinute`, `maxConnsPerHour`, `maxConnsPerDay`, and total equivalents;
- the Java server-side incoming connection filter / connection throttler placement;
- I2P+ equivalent server/stream admission placement;
- Yosemite 0.7.0 `SessionOptions`, stream session creation, and raw SAM option serialization behavior;
- the Emissary SAM implementation's actual accepted session/stream option support.

Do not assume that Java option names are valid in Emissary merely because SAM accepts arbitrary key/value text. Trace each candidate option to the code that consumes it.

Pin the exact external revisions used as evidence in the closure.

## 4. Scope and containment hierarchy

M088 MUST use the following decision hierarchy.

### Tier 1 — Existing `i2pcontrol` / Yosemite capability

If the already-pinned Yosemite API can express an Emissary-supported lower-layer admission option, wire only the necessary values from the existing `ServerAdmissionPolicy` into `AcceptedServerRuntimeConfig`/session creation.

Preferred production paths:

- `emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs`;
- `emissary-cli/src/i2pcontrol/backends/runtime/admission.rs` only if a small accessor/translation is required;
- existing accepted-server tests.

### Tier 2 — Narrow fork-local boundary exposure

If Emissary already implements the necessary lower-layer SAM/stream control but Yosemite 0.7.0 merely fails to expose the option, document the exact missing boundary first.

A minimal boundary adjustment may be proposed only if all of the following are true:

1. the receiving Emissary layer already implements the semantic;
2. the change is a mechanical option-plumbing exposure, not a new admission algorithm;
3. it can be isolated without broad router/core refactoring;
4. dependency ownership, lockfile impact, and source-containment impact are explicit;
5. no upstream interaction is required.

Because Yosemite is an external dependency, M088 itself does **not** pre-authorize vendoring, a crates.io patch, a new git dependency, or an external repository write. If any of those is required, stop and create a separate narrowly scoped dependency-boundary plan for explicit approval.

### Tier 3 — Unsupported lower-layer semantic

If Emissary does not actually implement the relevant lower-layer connection limit, or exposing it would require router/streaming algorithm changes outside the accepted Proposal 170 containment budget, do not widen M088.

Close M088 with a precise unsupported-capability finding that records:

- where the earliest enforceable existing boundary is;
- why application admission remains post-accept;
- what work remains bounded by existing runtime/task/session limits;
- what lower-layer work remains attacker-influenceable;
- why a broader router/core change was rejected as out of scope.

M089 must then treat this as a known residual hardening limitation rather than falsely claiming pre-accept enforcement.

## 5. Required invariant mapping

Before changing code, produce a source-level flow for one inbound accepted stream:

```text
remote I2P peer
 -> Emissary streaming/SAM acceptance
 -> Yosemite SAM connection/session state
 -> Yosemite Session<style::Stream>::accept()
 -> TrustedPeerIdentity
 -> ServerAdmissionState
 -> handler/local target
```

For each layer, record:

- what resource is allocated;
- whether remote identity is known;
- whether a concurrency/rate limit exists;
- whether the limit is per-peer, aggregate, or global;
- whether the limit acts before or after stream establishment;
- what happens on rejection/reset/drop;
- whether rejection emits any attacker-distinguishable behavior beyond protocol-required failure.

This map is part of M088 closure evidence even if no production change is feasible.

## 6. Lower-layer policy translation rules

If a supported lower-layer limit is available, translate existing policy rather than inventing a second unrelated policy surface.

Required principles:

- preserve `ServerAdmissionState` as defense in depth and handler/local-target authority;
- do not increase any lower-layer limit above the corresponding application global bound without a documented reason;
- do not silently convert per-Destination application semantics into per-IP or other clearnet identity semantics;
- do not derive limits from attacker-controlled header/application data;
- do not add Proposal 170 JSON-RPC fields for Java-only streaming controls;
- do not persist hidden runtime options that the backend cannot apply;
- rejection should be drop/reset/fail-closed according to the actually supported streaming/SAM semantic, not a custom timing-jitter response.

If only a global concurrent-stream limit is safely supported, implementing that alone may satisfy M088's production portion. Do not overengineer Java's entire connection-throttling matrix merely for nominal parity.

## 7. Required tests when implementation is feasible

Use the existing fake-SAM/unit boundary where possible. Tests must distinguish session-creation option plumbing from application admission.

At minimum prove:

1. the intended lower-layer option is emitted/applied during session creation or at the earliest supported pre-accept boundary;
2. the configured value is derived from the existing accepted-server policy and does not exceed its global concurrency bound without explicit rationale;
3. all accepted-stream server families receive the same common lower-layer protection unless a family has a documented incompatibility;
4. the application-level `ServerAdmissionState` remains active after lower-layer enforcement;
5. unsupported option/capability states fail truthfully rather than persist-and-ignore;
6. private server Destination material and full peer Destination text are not leaked in option/error diagnostics;
7. cancellation/restart creates no stale cross-generation limiter state;
8. no lock is held across network I/O;
9. no new dependency/default-feature leakage occurs.

If the lower-layer behavior cannot be exercised with deterministic local tests because the receiver is outside the current repository boundary, test the exact option serialization/plumbing and document the untestable external semantic instead of adding public-network infrastructure.

## 8. HTTP, IRC, and Streamr interaction

M088 changes only the common accepted-stream server layer.

It must not alter protocol-specific parsing or filtering in:

- `httpserver` / inbound `httpbidirserver`;
- `ircserver`;
- generic server payload relay beyond M087;
- Streamr datagram behavior.

Streamr does not use this accepted-stream path and is outside M088 production scope.

## 9. Explicit non-goals

M088 MUST NOT:

- replace the existing application admission algorithm;
- add process-wide cross-tunnel accounting;
- replace fixed-window rate accounting with token bucket/GCRA;
- add randomized delay/jitter/padding;
- add a parallel SAM implementation inside `i2pcontrol`;
- vendor or fork Yosemite without a new explicit plan;
- modify streaming/router algorithms merely to emulate Java I2P;
- broaden server target routing beyond loopback;
- change Proposal 170 fields/actions/types;
- reopen RouterInfo, AddressBook, or unrelated base-I2PControl work;
- add hosted CI/fuzz/soak/deanonymization infrastructure;
- interact with upstream repositories.

## 10. Ordered work packages

### A. Pin and map the actual boundary

1. Record implementation head and Yosemite version/revision.
2. Trace `Session<style::Stream>::new` and `accept()` through emitted SAM commands.
3. Trace Emissary's SAM/session handling for candidate streaming options.
4. Produce the layer/resource/limit map from Section 5.

### B. Select disposition

Choose exactly one evidence-backed branch:

- **supported directly** — implement Tier 1;
- **narrow boundary exposure exists but requires separately approved dependency work** — stop M088 production changes and create that plan;
- **unsupported without broad router/core changes** — record the limitation and keep application admission as the earliest in-scope bound.

Do not blend the branches or silently widen scope.

### C. Implement only the supported narrow translation

If Tier 1 is viable, add the minimal session option/plumbing plus deterministic tests.

### D. Re-run common accepted-server regressions

Verify generic, HTTP, HTTP-bidir inbound, and IRC accepted-stream paths still start/stop and retain application admission semantics.

### E. Close M088

Create `plans/closure/i2pcontrol-proposal-170/088-closure.md` with the boundary map, disposition, changed paths, tests, residual risk, and containment evidence.

## 11. Verification discipline

Required when production code changes:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol accepted_server
cargo test -p emissary-cli --no-default-features --features i2pcontrol admission
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

Add the smallest additional i2pcontrol test command needed to cover server-family startup/session creation if the test filters above do not exercise it.

When the disposition is evidence-only because lower-layer support is unavailable, verification is instead:

- source/revision evidence for every claimed boundary;
- changed-path review proving no production code was modified;
- M062 exact-path containment;
- `git diff --check`.

## 12. Acceptance criteria

M088 may close only when:

1. the earliest enforceable inbound-stream admission boundary is source-mapped;
2. Yosemite and Emissary support for candidate lower-layer options is established from code, not assumed from Java naming;
3. if a narrow supported lower-layer bound exists, it is applied before application-visible handler admission and tested as far as the repository boundary permits;
4. existing `ServerAdmissionState` remains in place as defense in depth;
5. no new Proposal 170 API surface is added;
6. no application identity is replaced with clearnet/IP identity;
7. unsupported capability is recorded truthfully rather than simulated with another post-accept counter;
8. no broad core/router/startup/dependency refactor occurs;
9. any need to vendor/fork/patch Yosemite stops this plan and produces a separate explicit dependency-boundary plan;
10. containment and focused tests pass for any code change;
11. `git diff --check` passes;
12. no upstream interaction occurs.

## 13. Stop conditions

Stop M088 and do not widen scope if:

- the candidate option is accepted textually but ignored by Emissary;
- the earliest meaningful limit requires a new streaming/router algorithm;
- Yosemite must be forked/vendored/patched to proceed;
- a parallel raw SAM implementation is proposed inside `i2pcontrol`;
- a solution requires broad `emissary-core/**` changes;
- a new dependency or default feature is required without separate approval;
- public-network load/deanonymization testing is proposed.

## 14. Closure evidence required

`plans/closure/i2pcontrol-proposal-170/088-closure.md` MUST include:

- baseline and final SHA;
- pinned Yosemite and external reference revisions;
- full inbound boundary/resource/limit map;
- exact candidate option support matrix;
- selected Tier 1/2/3 disposition;
- exact changed paths, if any;
- proof the application admission layer remains active;
- focused test commands/outcomes or evidence-only verification when no production change is feasible;
- M062 and `git diff --check` outcomes;
- residual lower-layer resource-exhaustion/correlation risk stated without euphemism;
- containment justification;
- explicit internal-only/no-upstream-interaction attestation.

## 15. Successor handoff

M089 becomes ready only after both M087 and M088 are closed. If M088 identifies a required separately approved dependency-boundary plan, M089 remains blocked until that plan is either closed or the lower-layer limitation is explicitly accepted as out of scope.
