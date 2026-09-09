# M152 — Final Residual Proposal-170 Requalification

Status: **deferred / unregistered; hard-depends on M151 closure and the then-current M146/provider disposition**

Class: invariant / qualification / final support authority

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-post-m146-corrective-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Promotion budget: **zero Proposal cells**.

## 1. Objective

Perform the final current-head whole-surface Proposal-170 requalification after the safe residual implementation chain reaches its terminal state.

M152 decides one of two truthful outcomes:

1. **full support for all applicable cells at the pinned revision**, only if there are zero applicable blockers; or
2. **safe partial / terminal under current security policy**, if all planned safe work is complete but one or more explicitly accepted blockers remain, including M146 `UseOutproxyPlugin` unless a separately accepted provider successor resolves it.

M152 is not an implementation milestone. Any required production change is a stop condition requiring a separate corrective plan.

## 2. Hard dependencies and entry conditions

Hard dependencies:

- M151 closed or otherwise reached a truthful terminal disposition under the current corrective roadmap;
- M153 closed and established the current runtime/security qualification authority;
- M154 and M147-M151 dispositions are recorded in the registry;
- M146 remains either closed blocked or is explicitly superseded by a separately accepted provider successor;
- no other Proposal capability plan is registered.

Entry begins from a mechanically recomputed M095 state. Do not assume predecessor maximum promotion budgets were achieved.

## 3. Production path budget

Production Rust, Cargo/dependency, Yosemite, runtime configuration, router, NetDB, crypto, transport and frontend changes are **forbidden** in M152.

Authorized work is limited to:

- final qualification/integration/adversarial tests;
- machine-evidence metadata reconciliation where mechanically stale;
- active docs/registry/roadmaps;
- final closure evidence.

If a runtime/code/dependency change is required, stop and create/register a separate corrective implementation plan.

## 4. Mechanical final matrix audit

At start:

- parse every M095 TunnelManager row;
- recompute total/apply/blocked/N/A counts;
- independently enumerate every blocked cell identity;
- verify every `apply` cell names real runtime evidence/closure ownership;
- verify every `not_applicable` cell has affirmative Proposal/reference family evidence where required;
- reject any capability represented only by parser/persistence/serializer/dormant state.

Exactly 840 cells must be accounted for.

### Full-support gate

A full-support claim requires:

- `blocked_primitive == 0`;
- every cell is `apply` or evidence-backed `not_applicable`.

### Safe-partial terminal gate

If blockers remain, M152 may still close the current safe residual workstream only when:

- every remaining blocker is explicitly named and tied to an accepted blocked/security disposition;
- no unimplemented capability is mislabeled complete;
- all implemented applicable cells requalify cleanly;
- active docs remain explicitly partial;
- the closure says what future architecture/security decision would be needed to resume.

If M146 remains unchanged, the expected terminal residual is at least the four `UseOutproxyPlugin` cells.

## 5. Whole-surface composition requalification

Re-run and cross-compose at least:

### Control plane

- finite token lifetime;
- bounded JSON-RPC body/request/task/batch admission;
- batch auth/notification semantics;
- fail-closed non-loopback management TLS;
- secret/token/password/private-key redaction.

### Tunnel/session lifecycle

- shared session/destination ownership;
- variance/backup/live quantity/LeaseSet desired-count behavior;
- Reduce -> Close -> authoritative `IdlePolicy` -> NewDest sequence;
- manual/restart/failure negative paths;
- cancellation and generation isolation.

### M140-M146 residual application features

- UniqueLocal source binding preserves loopback confinement;
- SSLProxies/JumpList preserve I2P-only egress and injection safety;
- Profile affects only the retained streaming family;
- UseSSL remains application TLS and cannot weaken management/SAM TLS;
- MultiHoming reply LeaseSet bundling preserves handshake/liveness/privacy rules;
- M146 blocked behavior still cannot create direct-clearnet fallback or accept-inert provider support.

### SigType/LeaseSet-security tail

For every actually completed predecessor:

- generated SigType identity must match selected suite with no fallback;
- encrypted LeaseSet cannot downgrade to plaintext;
- OptionalLookup cannot downgrade to public lookup or leak secret;
- LeaseSetClientAuths must reject unauthorized clients and cannot downgrade to unauthenticated publication.

If a predecessor closed blocked, test fail-before-allocation behavior instead of pretending support.

At least one integrated test must combine late features where interaction is plausible.

## 6. Containment and dependency requalification

M152 must independently prove:

- Proposal/admin policy remains rooted under `emissary-cli/src/i2pcontrol/**` wherever possible;
- every non-I2PControl production file introduced by the workstream is individually present in M061 with neutral owner/rationale;
- no broad path/glob exception was introduced for crypto/NetDB/I2NP convenience;
- no Proposal-specific terminology/API leaked into core;
- I2PControl-only direct dependencies remain optional and feature-owned;
- exact Yosemite fork remains optional/pinned unless separately superseded;
- feature-disabled/default builds do not activate I2PControl-only behavior/deps;
- no parallel raw SAM implementation exists;
- no direct-clearnet provider/fallback was introduced under M146 or any successor without an explicit accepted architecture/security change.

## 7. Security/adversarial closure matrix

Build a final table covering all sensitive residuals, including blocked ones:

- UniqueLocal: hostile peer/text cannot select target/non-loopback egress;
- proxy routing: hostile Host/proxy/jump values cannot cause DNS/direct clearnet/injection/unbounded cache;
- TLS: no plaintext fallback, cert/key leak, handshake stall or verification bypass;
- bundling: no wrong/private/fabricated LeaseSet;
- SigType: wrong type/key, fallback, malformed import, persistence mismatch;
- encrypted LeaseSet: wrong keys/type, downgrade, stale/fabricated leases;
- lookup: wrong secret, public-lookup downgrade, negative-cache flood;
- client auth: unauthorized/wrong-mode, auth-list flood, downgrade, rotation/restart race;
- UseOutproxyPlugin: no dummy/accept-inert provider and no direct-clearnet escape if still blocked.

No high/medium finding may be waived to improve completion status.

## 8. Authority/documentation transition

### If full support passes

Update:

- `plans/registry.md` to close the corrective/full-support roadmaps and make M152 current final Proposal-170 qualification authority;
- full-support roadmap status to complete for pinned revision;
- implementation README and active user-facing support docs to zero blockers/full support;
- exact final M095 counts/hash/head.

### If safe partial terminal passes

Update:

- registry to close the current safe residual workstream as partial/terminal under current policy;
- M152 as current final qualification authority for the implemented subset;
- full-support roadmap to remain partial with exact blocker list;
- implementation README/support docs with exact terminal blockers and required future architecture/security decision;
- no implication that blocked cells are supported.

Historical closure files remain unchanged in either outcome.

## 9. Verification baseline

At minimum:

```text
cargo check -p emissary-core
cargo check -p emissary-core --no-default-features --features no_std
cargo test -p emissary-core --lib --no-fail-fast
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

Also run every durable milestone-specific security/integration test from M127 onward that remains applicable to current head. M153 should have made historical aggregate assertions compatible with this broad-run requirement.

## 10. Acceptance criteria — full support

M152 may close **full/complete** only when:

- machine matrix has zero applicable blocked cells;
- every apply/N/A row is evidence-backed;
- whole-surface runtime/security composition passes;
- containment/dependency isolation passes with exact paths;
- no high/medium Proposal-scoped issue remains;
- required live/reference interoperability is recorded;
- active docs consistently state the same final matrix/revision;
- no production code changed during M152;
- external interaction remained read-only.

## 11. Acceptance criteria — safe partial terminal

M152 may close **partial/terminal under current policy** only when:

- all remaining blockers are exact, mechanically enumerated and tied to accepted blocked/security decisions;
- no further safe dependency-ready implementation plan exists under current architecture/policy;
- every implemented applicable cell requalifies cleanly;
- containment/security passes;
- active docs remain explicitly partial and name residual blockers;
- no production code changed during M152;
- external interaction remained read-only.

This disposition closes the current safe workstream but does **not** claim full Proposal support.

## 12. Stop conditions

Immediately stop and plan a corrective if:

- any current `apply` row is inert/approximate;
- any late feature weakens earlier security/containment invariants;
- production code/dependency change is required to pass qualification;
- interoperability contradicts local tests;
- exact path accounting requires a broad waiver;
- a blocker lacks a defensible accepted disposition;
- documentation cannot be reconciled without misstating support.

## 13. External-interaction boundary

All external specification/reference access is read-only. M152 authorizes no upstream issue/PR/review/contact/submission or external repository mutation.

## 14. Closure evidence required

Record final HEAD, exact M095 hash/counts/blocked identities, full-support versus safe-partial disposition, all verification outputs, M127-M151 composition/adversarial table, M146/provider status, live/reference evidence, final M061/M062 diff, active documentation diff, historical-closure immutability review, final qualification-authority decision, and explicit external read-only attestation.
