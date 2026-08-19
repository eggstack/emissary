# M084 — Merged-Head Integration and Planning Corrective

Status: ready — sole dependency-ready Proposal 170 tunnel-security handoff

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`.

Corrective predecessors / historical evidence:

- M077 closure: `plans/closure/i2pcontrol-proposal-170/077-closure.md`;
- M078 closure: `plans/closure/i2pcontrol-proposal-170/078-closure.md`;
- M079 closure: `plans/closure/i2pcontrol-proposal-170/079-closure.md`;
- M083 closure: `plans/closure/i2pcontrol-proposal-170/083-closure.md`.

Planning baseline: `e8feb9a3240a5a7b9dd5cc22a4ada47a0d9991ae`.

Classification: corrective pass / integration invariant.

## 1. Objective

Repair the narrow merge-integration defects created when the M077-M079 lineage and the later M083 lineage were merged, without changing Proposal 170 runtime behavior.

M084 exists because the individually verified branches no longer compose into a cleanly verifiable current head. It must restore a compilable test surface, restore containment bookkeeping, and reconcile all planning/status documents to the actual merged repository state. It must then leave M085 as the only final merged-head security reclosure handoff.

This is not a tunnel feature milestone and is not permission to redesign admission, HTTP, IRC, Streamr, TunnelManager, Yosemite integration, or router/core behavior.

## 2. Why prior verification missed this

M077, M078, and M079 were implemented and closed on the older tunnel-security branch lineage. M083 was implemented and closed later from the post-M082 planning lineage. Current `master` merges both histories.

Each branch passed its own focused/full checks before merge, but there is no recorded post-merge verification proving that the combined tree still compiles and satisfies the static planning guards. The merge retained incompatible assumptions from both sides:

1. M077's IRC admission-release test still calls the pre-M083 test helper `TrustedPeerIdentity::for_test("peer-destination")`, while current M083 trusted identity requires structurally valid Destination fixtures and no longer defines `for_test`.
2. The current M062 planning allowlist includes M083's closure but lost the M077/M078/M079 closure-path bookkeeping that existed on the older branch.
3. Planning surfaces disagree about whether M083, M077, M078, and M079 are ready, blocked, or closed.
4. The existing M079 closure audited the older pre-M083 final head and therefore cannot certify the current merged head.

A branch-local pass could not catch these merge-composition failures. M084 must add current-head regression evidence that would have caught them.

## 3. Canonical requirements and authority

M084 is governed by:

- `plans/000-long-term-specification.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`, especially corrective-pass, registry, Proposal 170, and internal-only rules;
- ADR-0001, ADR-0002, ADR-0003;
- M061 source containment and M062/M063 dependency/feature containment;
- the accepted M083 trusted-identity/admission contract;
- the retained M077 IRC and M078 Streamr runtime behavior.

The pinned Proposal 170 contract remains the `2026-05-20` revision.

External sources are read-only evidence only. No upstream issue, PR, review, submission, merge request, contribution preparation, or maintainer contact is authorized.

## 4. Invariants

M084 MUST preserve all current production behavior, including:

- exact Proposal 170 JSON-RPC fields/actions/types/statuses;
- all twelve currently implemented tunnel backend registrations;
- M083 exact trusted-Destination parsing and canonical downstream text;
- M083 32-byte Destination accounting identity;
- M083 peer-history/capacity/expiry-index semantics;
- M081 generic `leaseSetEncType` apply-or-reject behavior;
- M082 HTTP fixed-417 `Expect` rejection and canonical POST accounting;
- M076 request identity/proxy stripping and response fingerprint stripping;
- M077 five-second IRC local-target connect bound and ten-minute activity-resetting post-registration idle expiry;
- M078 loopback-only Streamr local UDP boundary, ten-subscriber ceiling, 60-second expiry, 15-second refresh, and payload/transport bounds;
- generation-local ephemeral state and existing stop/restart ownership;
- preferred production boundary under `emissary-cli/src/i2pcontrol/**`;
- no new `emissary-core/**` production path;
- no new dependency or Cargo feature widening.

## 5. Explicit non-goals

M084 MUST NOT:

- change admission limits, capacity formulas, counter semantics, or expiry policy;
- change trusted peer parsing/canonicalization semantics;
- add or remove tunnel types;
- change HTTP/IRC/Streamr protocol behavior;
- add generalized authentication, WAF, proxy, UDP, or routing machinery;
- change router algorithms or startup-service ownership;
- reopen RouterInfo 37/1/5 or M051 source-owner limitations;
- add CI, fuzzing, soak, benchmark, release, or public-network test infrastructure;
- rewrite historical closure records to pretend they audited commits they did not audit;
- perform or prepare any upstream interaction.

If a production runtime defect is discovered while executing M084, do not hide it inside this integration corrective. Record it and create a new narrow runtime corrective that blocks M085.

## 6. Required changes

### 6.1 Repair the stale IRC trusted-peer test fixture

In the `#[cfg(test)]` portion of `emissary-cli/src/i2pcontrol/backends/irc_server.rs`:

- replace the removed `TrustedPeerIdentity::for_test("peer-destination")` construction with the M083 structurally valid trusted-Destination fixture path;
- prefer the existing test-only re-export at `runtime::peer_identity::test_fixtures::distinct_peer` (or an equivalent already-existing structurally valid fixture);
- do not add a compatibility `for_test` helper that accepts arbitrary strings, because that would weaken the M083 test invariant;
- do not modify production IRC handler behavior.

Required regression: the admission-release-on-idle test must compile and exercise a structurally valid canonical peer identity under the current M083 API.

### 6.2 Restore M062 planning-path consistency

Update only the planning-path bookkeeping in `emissary-cli/tests/m062_dependency_containment.rs` so the guard recognizes the planning/closure files that actually exist after the merge.

At minimum account for:

- `plans/closure/i2pcontrol-proposal-170/077-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/078-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/079-closure.md`;
- `plans/implementation/i2pcontrol-proposal-170/084-merged-head-integration-and-planning-corrective.md`;
- `plans/implementation/i2pcontrol-proposal-170/085-merged-head-tunnel-security-reclosure.md`;
- M084 closure path when created.

Do not broaden glob rules or production-path exceptions. This is exact-path bookkeeping only.

The M062 guard must still reject unrelated production paths and retain all M061/M062/M063 ownership semantics.

### 6.3 Reconcile active planning state

Update the active planning surfaces so they agree on the following facts:

- M083 implementation/closure is present and accepted for its own scope;
- M077 and M078 implementations/closures are present in current history;
- historical M079 closure remains useful evidence for the older lineage but does not certify current merged `master`;
- M079's current-head final-closure authority is superseded by M085;
- M084 is the sole `ready` handoff until it closes;
- M085 is blocked on M084;
- the tunnel-security workstream remains `corrective pass required` until M085 independently accepts the post-M084 actual head.

Reconcile at minimum:

- `plans/registry.md`;
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- user-facing I2PControl support/status docs that currently claim the security phase is already closed or that an obsolete handoff is next.

Do not erase historical M077-M083 records. Correct current disposition without rewriting history.

### 6.4 Preserve historical M079 evidence truthfully

Do not edit `079-closure.md` to claim it audited M083 or the merged head. Current planning/docs should instead state that:

- M079 closed the older M077/M078 lineage at its pinned head;
- the later merge with M083 invalidated M079 as final current-head certification;
- M085 owns the new independent merged-head reclosure.

If a short status note is needed in the M079 implementation plan, it may identify M085 as the current-head successor, but historical requirements/evidence must remain intact.

## 7. Ordered work packages

### A. Establish exact current baseline

1. Record `git rev-parse HEAD` and confirm it descends from both the M077-M079 lineage and M083 closure lineage.
2. Record `git log --graph --oneline --decorate` for the relevant merge range.
3. Run the focused compile/test commands before edits and capture the actual failures; do not rely only on the planning review assertion.

### B. Repair test-only API integration

1. Replace the stale IRC trusted-peer fixture with an existing M083-valid fixture.
2. Add no production compatibility shim.
3. Run focused IRC tests immediately.

### C. Repair exact containment bookkeeping

1. Add only missing planning/closure paths.
2. Run M061 and M062 containment tests.
3. Confirm no new broad production authorization was introduced.

### D. Reconcile status documents

1. Normalize registry/roadmap/implementation README.
2. Correct user-facing support docs so security closure is pending M085.
3. Preserve unrelated Proposal 170 partial-support/source limitations.

### E. Verify merged integration

Run the full current-head package/static suite in Section 11.

### F. Close M084 and advance M085

Create `plans/closure/i2pcontrol-proposal-170/084-closure.md` only after all M084 evidence passes. The registry must then mark M084 closed and M085 as the sole `ready` handoff.

## 8. Failure, cancellation, restart, and contention semantics

M084 must not change runtime ownership, so the expected semantics are unchanged.

Verification must nevertheless ensure the merge did not regress:

- admission lease release after IRC idle expiry;
- handler cancellation/abort release;
- Streamr stop/restart generation isolation;
- HTTP accepted-handler cancellation;
- no mutex held across target connect, network I/O, sleeps, or joins.

Any failure in these production semantics is outside M084's permitted fix scope and requires a separate runtime corrective before M085.

## 9. Compatibility and migration

No external API, persistent schema, tunnel option, runtime default, destination format, or migration is authorized.

The only source-code behavior change expected is test-only fixture construction. M062 changes are static-guard metadata. Documentation changes correct status only.

A runtime-visible diff is a stop condition unless it is proven to be an incidental no-op formatting/import adjustment required by the test-only edit.

## 10. Focused regression evidence

Required focused evidence:

- IRC idle-release test compiles with M083-valid trusted identity;
- all focused `irc_server` tests pass;
- focused `runtime::admission` tests pass;
- focused trusted-peer tests pass, including trailing-byte rejection/canonical text;
- focused Streamr tests pass;
- focused HTTP tests pass;
- M061 containment passes;
- M062 containment passes and explicitly covers the merged closure/planning paths.

The M062 result must demonstrate that the exact bookkeeping repair, not a relaxed guard, made the suite pass.

## 11. Verification commands

Record exact outcomes for at least:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol irc_server
cargo test -p emissary-cli --no-default-features --features i2pcontrol runtime::admission
cargo test -p emissary-cli --no-default-features --features i2pcontrol peer_identity
cargo test -p emissary-cli --no-default-features --features i2pcontrol streamr
cargo test -p emissary-cli --no-default-features --features i2pcontrol http
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

Use repository-accepted scoped nightly rustfmt for touched Rust files. Do not use formatting drift elsewhere as a reason to edit unrelated files.

## 12. Documentation/static-guard acceptance

Before closure:

- registry has exactly one next handoff and no contradictory sentence/table;
- roadmap dependency graph is `M084 -> M085` for remaining tunnel-security work;
- implementation README agrees with registry;
- user-facing support docs do not claim final tunnel-security closure before M085;
- historical M079 remains identified as historical/pinned evidence, not silently deleted;
- M062 accepts all actual Proposal 170 planning/closure paths in the merged tree and still rejects unauthorized production paths.

## 13. Acceptance criteria

M084 may close only when:

1. the current merged head's I2PControl test target compiles;
2. no stale `TrustedPeerIdentity::for_test` or arbitrary-string trusted-peer fixture remains in the M077 IRC regression path;
3. the replacement fixture is structurally valid under M083 exact-Destination rules;
4. M061 and M062 pass without widening production-path authority;
5. M077/M078/M079 closure paths and M084/M085 planning paths are correctly represented by the static guard;
6. registry/roadmap/implementation README/user-facing support status agree;
7. M079 is retained only as historical older-lineage closure evidence;
8. M085 is the sole next final-reclosure handoff after M084;
9. full feature-enabled I2PControl tests pass;
10. feature-disabled and feature-enabled checks pass;
11. strict package Clippy passes;
12. no production runtime semantics changed;
13. no new dependency/core/startup-service scope was introduced;
14. no upstream interaction occurred.

## 14. Stop conditions

Stop and create a separate corrective plan if:

- fixing the IRC test requires weakening `TrustedPeerIdentity` production validation;
- any current production runtime test exposes a high/medium security, anonymity, correctness, lifecycle, or option-truthfulness defect;
- a core/Yosemite API change is required;
- containment can pass only by broadening allowed production globs;
- merge reconciliation requires changing Proposal 170 wire behavior;
- current head contains an unexplained production change outside the accepted containment boundary.

## 15. Closure evidence required

`084-closure.md` must include:

- exact pre/post implementation heads;
- proof of the two-parent merge lineage that caused the mismatch;
- before/after IRC fixture evidence;
- before/after M062 exact-path evidence;
- a planning/document reconciliation table;
- exact command outcomes;
- changed-path review proving no runtime behavior expansion;
- unresolved findings with severity;
- disposition for M085 readiness;
- explicit external-interaction attestation.

M084 closure does not close the tunnel-security workstream. It only creates a coherent merged head for M085 to audit independently.
