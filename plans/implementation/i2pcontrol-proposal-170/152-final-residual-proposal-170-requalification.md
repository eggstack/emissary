# M152 — Final Residual Proposal-170 Requalification

Status: **deferred / unregistered; hard-depends on M162 closure and any M161 outcome-A implementation successor**

Class: invariant / qualification / final support authority

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Promotion budget: **zero Proposal cells**.
Production budget: **zero production Rust/dependency/Yosemite changes**.

## 1. Objective

Perform the final whole-surface Proposal-170 requalification after the corrected LeaseSet-security line reaches its truthful terminal state.

M152 decides only between:

1. **full support** for the pinned Proposal revision, requiring zero applicable blockers; or
2. **safe partial / terminal under current policy**, when all safe implementation work is complete and every remaining blocker has an explicit accepted architecture/security disposition.

M152 cannot implement or promote capabilities.

## 2. Entry gate

Before registration:

- M153 remains accepted current-head qualification ancestry;
- M146 `UseOutproxyPlugin` is either still explicitly blocked or superseded by a separately accepted provider plan;
- M147/M148 configurable `SigType` is either still explicitly blocked or superseded by a separately accepted architecture/security plan;
- M155-M160 have closed with recorded dispositions;
- M161 has an explicit legacy-AES disposition;
- if M161 outcome A created a legacy-LS1 implementation successor, that successor has closed before M162 could promote `EncryptLeaseSet`;
- M162 has closed with the exact final LeaseSet-field promotions/blockers;
- no other Proposal capability plan is registered.

The entry matrix is mechanically recomputed; no predecessor promotion ceiling is assumed to have been achieved.

## 3. Mechanical matrix audit

Parse all 840 M095 TunnelManager cells and independently verify:

- exact apply/blocked/N/A totals;
- exact blocked cell identities;
- every `apply` row has real runtime/interoperability evidence rather than parser/persistence/serializer reachability;
- every N/A row has affirmative family/spec evidence;
- `SigType`, `UseOutproxyPlugin`, `EncryptLeaseSet`, `OptionalLookup`, and `LeaseSetClientAuths` dispositions match their actual closed milestones.

Full support requires `blocked_primitive == 0`.

Safe-partial terminal closure requires every blocker to be tied to an accepted blocked/security disposition and no remaining dependency-ready safe implementation plan under current architecture/policy.

## 4. Whole-surface composition requalification

Re-run and cross-compose:

- M127 finite token lifetime;
- M128 bounded JSON-RPC admission/batches;
- M129 fail-closed management TLS;
- shared session/destination ownership and M135 live quantity/LeaseSet truthfulness;
- Reduce -> Close -> authoritative IdlePolicy -> NewDest lifecycle;
- M141 UniqueLocal loopback confinement;
- M142 SSLProxies/JumpList I2P-only egress/injection safety;
- M143 retained Profile behavior;
- M144 application TLS separation;
- M145 reply-LeaseSet bundling privacy/liveness;
- M146 blocked provider behavior if still blocked;
- M156 Red25519/blinding and M157 encrypted-LS2 publication if implemented;
- M158 lookup-secret/addressing;
- M159 PSK auth;
- M160 DH auth;
- M161 legacy AES disposition/implementation result;
- M162 Proposal field validation, secret transactionality and no-downgrade semantics.

Blocked predecessors are tested for exact fail-before-allocation behavior, not treated as support.

## 5. Security/containment audit

Independently prove:

- Proposal/admin policy remains under `emissary-cli/src/i2pcontrol/**` wherever possible;
- every non-I2PControl production path introduced by the line is exact in M061/M062 with a neutral owner/rationale;
- no broad crypto/netdb/i2np/destination waiver exists;
- no Proposal terminology leaked into core neutral APIs;
- I2PControl-only direct dependencies remain optional/feature-owned;
- Yosemite remains exact optional authority unless separately superseded;
- no plaintext/unsecreted/unauthenticated downgrade exists in LeaseSet security modes;
- Red25519/PSK/DH/private lookup material is not logged or response-facing;
- no direct-clearnet fallback was introduced for M146;
- no high/medium security finding is waived to improve completion status.

## 6. Expected terminal examples

From the M153 baseline `336/29/475`:

- if modern LeaseSet security allows `OptionalLookup` + `LeaseSetClientAuths` promotion but legacy AES keeps `EncryptLeaseSet` blocked, a possible terminal state is `346/19/475`;
- if all ten `EncryptLeaseSet` values also close, a possible state is `351/14/475`;
- if later independent work also resolves SigType and UseOutproxyPlugin, full support may become possible.

These are examples only; M152 must use the actual machine matrix.

## 7. Verification baseline

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

Also run all durable security/interoperability tests created by M156-M162.

## 8. Stop conditions

Stop and create a corrective implementation plan if:

- any current `apply` cell is inert/approximate;
- production code/dependency changes are required;
- interoperability contradicts local tests;
- a late feature weakens earlier containment/security;
- exact path accounting requires a broad waiver;
- any blocker lacks an accepted disposition.

## 9. Closure transition

If full support passes, make M152 the final qualification authority and close the full-support roadmap at the pinned revision.

If safe partial passes, make M152 the final qualification authority for the implemented subset, keep the full-support roadmap explicitly partial, enumerate every terminal blocker and the architecture/security decision required to resume.

Historical closures remain immutable. External reference access remains read-only.

## 10. Closure evidence

Record final HEAD, exact M095 hash/counts/blocked identities, full-vs-partial disposition, verification outputs, M127-M162 composition/adversarial table, M061/M062 evidence, live/reference interoperability, active documentation reconciliation, unresolved blockers, and external read-only attestation.