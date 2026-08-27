# ADR-0004: Pinned Full Proposal 170 Completion and Minimal-Core Boundary

Status: accepted

Date: 2026-08-27

Decision owners: project maintainers

Supersedes, only for the newly authorized full-support completion phase:

- the statements in `plans/000-long-term-specification.md`, `plans/002-long-term-roadmap.md`, and `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` that treat RouterInfo `37 available / 1 neutral / 5 unavailable` as an acceptable final source state;
- the statements that retain Proposal 170 `AddressBook.SetConfig` limitations as an acceptable final state;
- the statements that permit an applicable Proposal 170 TunnelManager runtime option to remain permanently rejected after all twelve tunnel backends are real.

ADR-0001, ADR-0002, and ADR-0003 remain accepted authority for exact contract spelling, startup/control-plane ownership separation, secret ownership, tunnel runtime/filter architecture, security boundaries, containment, and internal-only work unless this ADR explicitly changes a completion target.

Related planning:

- `plans/000-long-term-specification.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`.

Pinned external authority:

- I2P Proposal 170, `I2PControl Expansion`, status `Open`, revision created/updated `2026-05-20`.

## Context

The internal fork now has a mature Proposal 170 implementation:

- exact JSON-RPC/authentication composition for the Proposal 170 surface;
- operational AddressBook CRUD and subscription replacement;
- all twelve declared TunnelManager types backed by real control-plane runtimes;
- all seven canonical TunnelManager actions;
- operational ClientServicesInfo selectors;
- a RouterInfo matrix of 43 additions with 37 available, 1 protocol-permitted neutral value, and 5 truthfully unavailable rows;
- strong source/dependency containment and independently reclosed tunnel security at M093;
- no current production/security corrective handoff after M094.

That state is deliberately truthful but still partial. Maintainer direction on 2026-08-27 now authorizes a third phase whose target is full support for the pinned Proposal 170 revision rather than accepting the remaining source/configuration/option limitations indefinitely.

The proposal remains open. Therefore `full Proposal 170 support` in this repository always means full support against the explicitly pinned `2026-05-20` revision, not a claim that future proposal changes are already implemented or that upstream has reviewed or accepted this fork.

## Decision drivers

- Complete the pinned Proposal 170 surface without expanding into unrelated base I2PControl methods.
- Preserve the heavily reviewed Emissary router code wherever possible.
- Keep Proposal 170 administrative policy, samplers, configuration semantics, option mapping, and network/application adapters under `emissary-cli/src/i2pcontrol/**`.
- Permit lower-layer changes only where a canonical runtime owner is the only truthful source for a required fact.
- Avoid fabricating values merely to make the conformance matrix green.
- Avoid adding router algorithms or behavior solely to satisfy a telemetry selector.
- Finish runtime semantics for applicable TunnelManager options now that all twelve data planes are real.
- Implement `AddressBook.SetConfig` with security-preserving path confinement rather than arbitrary remote filesystem authority.
- Require real interoperability evidence before changing documentation to a full-support claim.
- Keep verification focused and proportional; no new CI/fuzz/coverage farm is required.
- Preserve the internal-only/no-upstream boundary.

## Considered options

### Option A — Keep the accepted partial state permanently

Rejected by current maintainer direction.

The 37/1/5 RouterInfo state, `SetConfig` rejection, and apply-or-reject option matrices remain safe intermediate states, but they no longer define the intended end state.

### Option B — Implement every missing surface by broadening `emissary-core`

Rejected.

Most remaining work is administrative or application-layer behavior. Broad core changes would enlarge the audited surface for API convenience and conflict with the containment strategy that has successfully kept Proposal 170 policy local.

### Option C — Return empty/zero/default values for every missing selector

Rejected.

A type-correct value is not automatically truthful. Empty collections, zero error codes, empty news, or other defaults may be used only where the implementation can prove that the value represents actual router state, not absence of an owner.

### Option D — Complete local semantics first, then add only minimal neutral owner facts where unavoidable

Accepted.

The completion sequence must exhaust the I2PControl-local implementation space before touching router-core ownership. Any lower-layer addition must expose a neutral bounded fact, not Proposal 170 policy or wire codes.

## Decision

### 1. Full-support target

The target end state is:

> Emissary fully supports I2P Proposal 170 against the pinned 2026-05-20 revision, with all applicable specified methods, selectors, configuration keys, tunnel actions/types/options, and service selectors implemented truthfully and operationally.

The word `applicable` permits a per-tunnel option cell to be classified `not_applicable` only when the pinned contract/reference semantics do not apply that option to that tunnel family. It does not permit a behaviorally meaningful applicable option to remain `unsupported` in the final matrix.

Unrelated base I2PControl methods such as `GetKeys`, `GetRate`, `RouterManager`, `NetworkSetting`, and `AdvancedSettings` remain outside this Proposal 170 completion phase unless the pinned proposal itself changes.

### 2. Phase ordering

Implementation MUST proceed in this order:

1. establish one exact full-support conformance/applicability/source matrix and path budget;
2. complete work that can stay under `emissary-cli/src/i2pcontrol/**`;
3. only then implement missing canonical router-owner facts with minimal neutral core changes where unavoidable;
4. perform integrated live interoperability and full-conformance reclosure.

Future milestones may be written in advance for handoff continuity, but only dependency-ready work is registered as executable according to `plans/003-planning-process.md`.

### 3. Preferred production boundary

Proposal 170-specific code SHOULD remain under:

`emissary-cli/src/i2pcontrol/**`

This includes:

- request-independent transit-bandwidth sampling derived from an existing authoritative cumulative counter;
- router-news retrieval/cache/parsing if required by the pinned contract;
- AddressBook configuration interpretation, persistence, path confinement, refresh/publish/proxy policy, and administrative metadata;
- TunnelManager option applicability/capability mapping and runtime translation;
- Proposal 170 wire-code mapping for neutral router observations.

A new standalone crate is not required merely for aesthetic isolation.

### 4. RouterInfo transit 15-second metric

The prior request-local implementation was invalid because API request history determined the metric. The full-support phase may add an optional I2PControl-owned periodic sampler when the `i2pcontrol` feature/service is enabled, provided:

- it reads the already-authoritative cumulative transit-byte counter;
- sampling cadence is independent of RouterInfo requests;
- memory/task state is bounded;
- no router-core timer or traffic-path instrumentation is added;
- feature-disabled/default execution creates no sampler.

### 5. Router news

The full-support phase may add a bounded I2PControl-owned router-news source using the pinned/reference semantics. It must define source authenticity, fetch/refresh bounds, cache size, failure behavior, and privacy implications before returning data.

No news parser/fetcher belongs in `emissary-core` solely for Proposal 170.

### 6. Network-error owner

`i2p.router.net.error` and `.error.v6` require actual error-reason state. The final implementation MUST NOT infer `0 / No error` merely because no source exists.

If the current transport/reachability owners already know the relevant reason, the smallest neutral state may be surfaced through existing accepted inspection/event paths. Core state must use implementation-neutral terminology; Proposal 170/i2pd integer mapping remains in `i2pcontrol`.

Any such change must be passive and must not alter transport retry, peer selection, reachability testing, tunnel building, or routing behavior.

### 7. Banned-peer semantics

The completion phase MUST first perform an exhaustive source/semantic audit.

If Emissary has an existing enforceable peer-ban/exclusion concept, Proposal 170 may expose a bounded read-only view through the canonical owner.

If Emissary has no state in which a peer can be banned by design, the implementation may return an empty banned-peer map only after the milestone proves that this is a semantic statement about actual router capability/state rather than an unowned fallback. Do not create a new peer-ban algorithm solely to populate a telemetry getter.

If the pinned proposal cannot be satisfied truthfully without introducing substantive new router ban behavior, the full-support phase is blocked at that row pending a separate maintainer architecture decision; the implementation must not silently broaden routing/security behavior.

### 8. AddressBook SetConfig

All thirteen pinned SetConfig keys require explicit operational disposition.

Behaviorally meaningful keys must be applied. Path-valued keys must use an explicit configured administrative root and normalized/path-confined semantics; remote requests must not gain arbitrary filesystem read/write authority. Relative paths are permitted only when their normalized resolution stays within an authorized root.

`theme` may remain administrative metadata with no router/frontend effect because frontend work is outside scope, but it must round-trip deterministically if accepted. The implementation must distinguish harmless metadata from controls that affect downloader, publication, or file ownership.

### 9. TunnelManager option completeness

The current fail-before-allocation capability matrix remains the safe intermediate rule. The full-support target changes only the end state:

- every pinned option/type cell is explicitly classified;
- applicable options are implemented and applied;
- truly irrelevant cells are `not_applicable` with reference/spec rationale;
- no applicable cell remains `unsupported` in the final full-support matrix;
- security-sensitive file/key/password options use path confinement or backend-owned secret storage and remain redacted;
- existing HTTP/IRC/Streamr security boundaries remain mandatory and are not weakened for option parity.

If Yosemite/SAM lacks a required session primitive, the owning milestone must stop and record the exact missing primitive. It must not add a Proposal-170-shaped core API as a convenience shortcut.

### 10. Containment ceiling

M061 remains the accepted source-boundary authority. Existing non-I2PControl production paths are a ceiling, not an invitation to spread new policy.

A full-support milestone may modify an existing allowed lower-layer path only when the plan names it explicitly and demonstrates why the canonical owner must change. A new non-I2PControl production path requires an explicit containment amendment before implementation.

### 11. Interoperability requirement

The repository must not change its support statement to `full` based only on parsers, unit tests, or local loopback composition.

Final closure requires bounded live evidence against a reseeded I2P environment and at least one reference router where practical, covering representative RouterInfo, AddressBook, all twelve TunnelManager families, ClientServicesInfo, persistence/restart, server destination stability, and data-plane traffic.

This is a focused acceptance harness, not authority for a new hosted CI matrix.

### 12. Proposal revision pin

Because Proposal 170 is still open, all matrices, plans, closures, and public support statements MUST name the pinned `2026-05-20` revision. A later proposal revision triggers a new delta audit; it does not silently invalidate historical closure evidence.

### 13. Internal-only rule

All repository writes remain internal to `eggstack/emissary`.

External I2P, I2P+, i2pd, Java I2P, go-i2p, Yosemite, specifications, issues, commits, and pull requests are read-only evidence. No upstream issue/PR/review/submission/merge/adoption request, contribution preparation, or maintainer contact is authorized.

## Consequences

### Positive

- the project gains a clear path from truthful partial support to a precise full-support claim;
- most new work remains isolated in the optional I2PControl subsystem;
- lower-layer changes are deferred until local implementation options are exhausted;
- the final claim is backed by an exact applicability matrix and live evidence;
- prior security and containment closures remain valid historical authority rather than being rewritten.

### Negative

- `i2pcontrol` gains additional background/configuration machinery for transit sampling and news;
- full TunnelManager option parity is materially larger than the completed twelve-type data-plane work;
- one or more RouterInfo rows may still reveal a genuine architecture blocker rather than a simple implementation task;
- AddressBook path/config parity requires careful filesystem and migration semantics.

## Compatibility and migration

- no Proposal 170 public method/type/action/key is renamed or extended;
- persisted TunnelManager definitions remain schema-compatible where possible; schema changes require versioned additive migration;
- AddressBook configuration gains operational persistence and may require a versioned migration from the current explicitly-rejected state;
- default/feature-disabled Emissary remains unaffected;
- startup-managed resources remain separate from control-plane-owned resources.

## Security and reliability implications

- remote administrative configuration remains authenticated and path-confined;
- news retrieval must not become an unbounded or unauthenticated content ingestion path;
- background samplers/fetchers are bounded and cancellable;
- network-error observation is passive and cannot mutate transport decisions;
- no new ban behavior is introduced solely for a getter;
- secret/key options remain backend-owned or confined and redacted;
- existing server admission/filtering invariants remain mandatory.

## Verification

The full-support roadmap must establish evidence that:

1. one machine-readable matrix covers every pinned Proposal 170 addition and every TunnelManager option/type applicability cell;
2. AddressBook SetConfig has no accepted-but-inert behaviorally meaningful key;
3. all applicable TunnelManager option cells are applied;
4. transit 15-second sampling is request-independent;
5. news has a real bounded source or the milestone remains blocked;
6. network-error values come from canonical explicit state;
7. banned-peer output is backed by actual ban/exclusion semantics or a proven by-design empty state;
8. no unplanned non-I2PControl production path is added;
9. live interoperability evidence exists before the full-support label is used;
10. no upstream interaction occurred.

## Supersession

ADR-0004 changes the intended final completeness target only. It does not invalidate earlier partial-support closures at the revisions they reviewed. Those records remain truthful historical evidence for the state before this newly authorized phase.