# M152 — Final Residual Proposal-170 Requalification

Status: **deferred / unregistered; hard-depends on M151 closure**

Class: invariant / qualification / final support authority

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Promotion budget: **zero Proposal cells**.

## 1. Objective

Perform a final, current-head, whole-surface Proposal-170 requalification after the residual primitive line closes. M152 decides whether the fork may truthfully move from **partial** to **complete for all applicable Proposal-170 cells at the pinned revision**.

M152 is not an implementation milestone. Any production defect or remaining primitive gap discovered here is a stop condition requiring a corrective plan.

## 2. Hard dependency and entry conditions

Hard dependency:

- M151 closed.

Before M152 registration, the registry must show M140-M151 dispositions/closures and no other registered Proposal capability plan.

Entry requires a mechanically recomputed M095 baseline from the then-current repository. Do not assume all preceding plans achieved their maximum promotion budgets. M152 starts from whatever truthful machine state exists.

## 3. Final support rule

Full Proposal-170 support may be claimed only if M152 proves:

1. exactly 840 TunnelManager option/family cells are accounted for;
2. every cell is `apply` or affirmative-evidence `not_applicable`;
3. `blocked_primitive == 0` for all applicable residuals;
4. every `apply` cell has real runtime behavior, not parser/persistence/wire-only reachability;
5. every `not_applicable` row has pinned Proposal/reference family evidence;
6. all canonical TunnelManager data planes/actions, RouterInfo additions, AddressBook and ClientServicesInfo surfaces remain qualified;
7. M127-M129 control-plane security invariants remain green;
8. M135-M137/M134 lifecycle behavior remains green;
9. M140 applicability corrections remain source-valid;
10. M141-M151 security/runtime behavior composes without cross-capability downgrade;
11. M061/M062 exact containment/dependency isolation is truthful on the final head;
12. no high/medium Proposal-scoped correctness or security defect remains.

If any condition fails, support remains partial.

## 4. Production path budget

Production Rust, Cargo/dependency, Yosemite and runtime configuration changes are **forbidden** in M152.

Authorized work is limited to:

- qualification/integration tests;
- current-head machine evidence reconciliation where a stale metadata field is proven wrong;
- active docs/registry/roadmaps;
- final closure evidence.

If a runtime/code change is required, stop and create/register a separate corrective milestone. Do not convert M152 into a cleanup implementation pass.

## 5. Mechanical matrix and residual audit

At start:

- parse all M095 rows and recompute counts/total;
- independently derive every `blocked_primitive` row;
- compare against M140-M151 closure deltas rather than trusting roadmap budgets;
- verify each `not_applicable` row has an evidence authority, including all M131/M140 reclassifications;
- verify each late `apply` row names the closure/runtime evidence that actually implemented it;
- reject any `apply` row whose runtime consumer is only Yosemite serialization or dormant state.

A zero-blocker prose claim is insufficient without the mechanical row audit.

## 6. Whole-surface composition requalification

Re-run and cross-compose at least:

### Control plane

- finite token lifetime and reachable expiration;
- bounded JSON-RPC body/request/task/batch admission;
- per-element batch auth and notification semantics;
- fail-closed non-loopback management TLS with no plaintext fallback;
- secret/token/password/private-key redaction.

### Tunnel/session lifecycle

- shared session/destination ownership;
- variance/backup and live quantity/LeaseSet desired-count behavior;
- Reduce* -> Close* -> authoritative IdlePolicy -> NewDest sequence;
- manual/restart/failure negatives;
- cancellation and generation isolation.

### Residual application features

- UniqueLocal source binding preserves loopback confinement;
- SSLProxies/JumpList preserve I2P-only egress and HTTP injection safety;
- Profile affects only applicable streaming families;
- UseSSL remains application TLS and cannot weaken management/SAM TLS;
- outproxy provider cannot create direct-clearnet fallback.

### Residual lower-layer/security features

- MultiHoming/reply bundling never leaks wrong/private/expired LeaseSet;
- SigType generated identity actually matches selected suite with no fallback;
- EncryptLeaseSet cannot downgrade to plaintext;
- OptionalLookup cannot downgrade to public lookup or leak secret;
- LeaseSetClientAuths rejects unauthorized clients and cannot downgrade to unauthenticated publication.

At least one integrated test should combine late features where interaction is plausible, rather than testing every primitive only in isolation.

## 7. Containment and dependency requalification

M152 must independently re-prove:

- Proposal/admin policy remains rooted under `emissary-cli/src/i2pcontrol/**`;
- every final non-I2PControl changed file is individually present in M061 with a neutral owner/rationale;
- no broad path/glob exception was introduced for crypto/NetDB/I2NP convenience;
- no Proposal-specific terminology/API leaked into core;
- I2PControl-only direct dependencies remain optional and feature-owned;
- exact Yosemite fork remains optional and pinned to accepted revision unless a separately accepted Yosemite closure explicitly supersedes it;
- feature-disabled/default builds do not activate I2PControl-only deps or behavior;
- no parallel raw SAM implementation exists.

## 8. Security/adversarial closure matrix

Build a final table mapping each sensitive residual to its adversarial evidence:

- local source binding: hostile peer identity/text cannot select target or non-loopback egress;
- proxy routing: hostile Host/proxy/jump values cannot cause DNS/direct clearnet/injection/unbounded cache;
- TLS: plaintext fallback, cert/key leak, handshake stall, verification bypass;
- bundling: wrong destination, expired/private LeaseSet, message-size boundary;
- SigType: wrong type/key, fallback, malformed import, persistence mismatch;
- encrypted LeaseSet: wrong keys/type, downgrade, stale/fabricated leases;
- lookup: wrong secret, public-lookup downgrade, negative-cache flood;
- client auth: unauthorized/wrong-mode, auth-list flood, downgrade, rotation/restart race.

No high/medium finding may be waived to achieve final completion.

## 9. Documentation/authority transition

Only after all qualification gates pass:

- update `plans/registry.md` to mark the residual roadmap closed complete and M152 as current final Proposal-170 qualification authority;
- update full-support roadmap status from partial to complete for the pinned revision;
- update `plans/implementation/i2pcontrol-proposal-170/README.md`;
- update active user-facing I2PControl support docs and AGENTS current-state wording;
- preserve all historical closure wording as historical evidence;
- record exact final matrix counts and pinned Proposal revision/hash.

If any blocker remains, keep all active docs explicitly partial and name the residual/corrective next handoff.

## 10. Failure/cancellation/runtime policy

M152 adds no runtime mechanisms. It must verify existing ones under composition and fail closed on evidence gaps.

Tests must use deterministic/fake clocks/network fixtures where possible rather than wall-clock sleeps, and bounded live/reference interoperability only where necessary.

A flaky or unavailable external reference service is not grounds to fabricate closure; record the blocked evidence and keep support partial if the required interoperability proof cannot be obtained.

## 11. Verification baseline

At minimum:

```text
cargo check -p emissary-core
cargo test -p emissary-core --no-fail-fast
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Also run every durable milestone-specific security/integration test from M127 onward that remains applicable to current head. The registered M152 plan should enumerate their exact test targets after M151 closure.

## 12. Acceptance criteria

M152 closes **complete** only when:

- machine matrix mechanically has zero applicable blocked cells;
- every apply/N/A row is evidence-backed;
- whole-surface runtime/security composition passes;
- containment/dependency isolation passes with exact paths;
- no high/medium Proposal-scoped issue remains;
- required live/reference interoperability is recorded;
- active docs consistently state the same final matrix and pinned revision;
- no production code was changed during M152;
- external interaction remained read-only.

Otherwise M152 closes **blocked/corrective-required** and the repository remains partial.

## 13. Stop conditions

Immediately stop and plan a corrective if:

- any current `apply` row is discovered inert/approximate;
- any applicable blocker remains;
- any late feature weakens earlier security/containment invariants;
- production code/dependency change is required to pass qualification;
- final interoperability evidence contradicts local tests;
- exact path accounting cannot be reconciled without a broad waiver.

## 14. Closure evidence required

Record final HEAD, exact M095 hash/counts and row audit, zero-blocker proof or named residuals, all verification outputs, M127-M151 composition/adversarial table, live/reference evidence, final M061/M062 path/dependency diff, active documentation diff, historical-closure immutability review, current qualification authority decision, and an explicit statement that no external/upstream mutation/contact/submission occurred.