# M153 — Post-M146 Current-Head Requalification and Authority Rebase

Status: **registered / dependency-ready**

Class: invariant / qualification / corrective

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Corrective predecessors:

- M139 post-lifecycle integrated requalification;
- M140 residual streaming applicability re-freeze;
- M141-M145 capability closures;
- M146 blocked outproxy-provider feasibility closure;
- M145 follow-up commit `7cbd80a6d72aa07d158ba9dc74f8bbacef767be2` fixing the no-std `String` import and formatting after the original M145 closure commit.

Planning baseline:

- repository head at registration: `a0c4a791a6a7a974d34eaf93c45330aafd11116f`;
- last production-bearing head: `7cbd80a6d72aa07d158ba9dc74f8bbacef767be2`;
- current M095 matrix: `336 apply / 29 blocked_primitive / 475 not_applicable` across 840 TunnelManager option/family cells;
- residual blockers: 10 `SigType`, 15 encrypted/authenticated LeaseSet, 4 `UseOutproxyPlugin`;
- M139 remains the last whole-surface runtime/security qualification authority but predates M141-M145 production work;
- M146 closed blocked with zero production delta and zero promotions.

Promotion budget: **zero Proposal cells**.

## 1. Objective

Establish a truthful current-head integrated qualification baseline after M140-M146 before any cryptographic destination/LeaseSet work begins.

M153 corrects four concrete post-M139 qualification defects:

1. M139 is still named as the current runtime/security authority even though M141-M145 added production behavior after it;
2. M095 `current_production_head` still points to the pre-M141 lifecycle head instead of the accepted post-M145 production head;
3. the broad I2PControl test invocation is red because historical milestone suites assert obsolete current matrix totals/wording instead of preserving milestone-local facts independently from current-head authority;
4. M145 required the follow-up `7cbd80a...` no-std/format correction after its closure record, so the final accepted production state must be requalified explicitly.

M153 is qualification/test/documentation work only. It must not add or modify production behavior.

## 2. Hard dependencies and authority

Hard dependencies:

- M146 closure exists and is accepted as blocked;
- M141-M145 closures exist;
- current machine matrix mechanically recomputes to `336/29/475`;
- no Proposal capability successor is registered concurrently.

Historical closures remain immutable evidence. M153 supersedes M139 only for current-head runtime/security qualification if M153 closes cleanly.

Pinned external authority remains read-only:

- Proposal 170 revision `2026-05-20`, status Open;
- Java I2PControl reference head `45bb593000408071dd376b78848fdc246dccd964`;
- Java I2P/I2PTunnel snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`;
- Yosemite exact optional revision `59140a2277bf296928d2e8ce39a148182eeff044`.

## 3. Production path budget

Production Rust, Cargo manifests, Cargo.lock, Yosemite, router, NetDB, crypto, transport, startup-tunnel, frontend and workflow changes are **forbidden**.

Authorized work is limited to:

- tests under `emissary-cli/tests/**` when required to separate historical snapshot assertions from durable current-head assertions;
- M095/M105/M110/M061/M062 machine/planning metadata only when mechanically stale;
- active Proposal-170 planning/registry/roadmap/index docs;
- user-facing Proposal-170 support documentation and AGENTS current-state wording where necessary to align authority/counts;
- a new M153 closure record.

If any runtime or security defect requires production code, stop M153 and create a separate corrective implementation plan. Do not repair production under a qualification milestone.

## 4. Matrix and production-head reconciliation

Mechanically parse M095 and prove:

- total = 840;
- apply = 336;
- blocked_primitive = 29;
- not_applicable = 475;
- residual blocker identities are exactly 10 `SigType`, 15 LeaseSet-security, 4 `UseOutproxyPlugin`.

Then reconcile metadata:

- `current_production_head` must identify the last accepted production-bearing commit, currently expected `7cbd80a6d72aa07d158ba9dc74f8bbacef767be2` unless the audit proves a later production-bearing commit;
- M146's closure commit must not be called a production head because it made no production change;
- historical matrix hashes/counts in immutable closure records remain historical and are not rewritten.

No M095 cell disposition may change in M153.

## 5. Historical-test versus current-head test model

The full I2PControl suite currently contains historical milestone tests whose matrix-count assertions were useful at closure but now fail as later milestones legitimately changed M095.

M153 must classify each failing M126-M146-era suite assertion as one of:

1. **historical invariant** — preserve by asserting against that milestone's immutable closure/snapshot evidence or milestone-local cell facts rather than current aggregate counts;
2. **durable current-head invariant** — update to current M095 semantics and keep it in the ordinary suite;
3. **obsolete duplicate** — remove/replace only if another durable guard provides equal or stronger coverage and the historical closure still records the fact.

Do not silence failures with `#[ignore]`, broad feature gates, loose `>=` count checks or by deleting security assertions.

Add one explicit M153 current-head qualification test that owns the aggregate `336/29/475` assertion and exact 29-cell residual set.

## 6. Whole-surface post-M145 composition requalification

Re-prove the M139 baseline plus every post-M139 accepted change:

### M140 applicability

- six constructor-overridden/UDP `Profile` cells remain affirmative `not_applicable`;
- `ConnectDelay:streamrclient` remains affirmative `not_applicable`;
- `Profile:client` remains the only retained Profile cell.

### M141 UniqueLocalAddressPerClient

- only `httpserver`/`httpbidirserver` apply;
- canonical peer hash drives deterministic source address;
- target remains literal loopback;
- no DNS/non-loopback fallback;
- IPv6 unavailable-source behavior fails closed.

### M142 HTTP SSLProxies / JumpList

- I2P-only outproxy selection remains bounded;
- last-failure behavior remains bounded/deterministic;
- JumpList is response/address-helper metadata only and is never fetched;
- no host/header value can create direct clearnet/DNS escape or response injection.

### M143 Profile

- `bulk`/omitted preserve effective default;
- `interactive` reaches the neutral streaming max-window owner;
- generation/shared-session behavior remains deterministic;
- core stays Proposal-free;
- no-std build for touched core remains green.

### M144 UseSSL

- local listener TLS and local-target TLS remain distinct from M129 management TLS and Yosemite SAM TLS;
- real handshake occurs before/after the correct application boundary;
- trust verification remains fail-closed;
- no plaintext fallback;
- private TLS material remains redacted;
- timeout/cancellation/generation semantics remain bounded.

### M145 MultiHoming / shouldBundleReplyInfo

- exact neutral owner remains `SessionManager`;
- omitted/true bundle behavior and false suppression remain effective;
- `NewSession` mandatory handshake bundling remains intact;
- no wrong/private/fabricated LeaseSet is bundled;
- no-std compilation passes on final post-correction source;
- explicitly record `7cbd80a...` as part of accepted M145 production evidence.

### M146 blocked behavior

- all four `UseOutproxyPlugin` cells remain blocked;
- supplied values fail before allocation;
- ordinary ProxyList behavior remains unchanged when option omitted;
- no direct-clearnet provider or dummy provider exists;
- M146 contributes no production delta.

## 7. Security/control-plane regression

Re-run and preserve at least:

- M127 finite token lifetime;
- M128 bounded JSON-RPC batch/body/task semantics;
- M129 fail-closed non-loopback management TLS;
- M135-M137/M134 Reduce -> Close -> `IdlePolicy` -> NewDest composition;
- M061/M062 containment and dependency isolation;
- secret/password/token/private-key redaction;
- feature-disabled/default dependency isolation.

If any of these now fails behaviorally rather than from stale wording/counts, M153 stops corrective-required.

## 8. Containment and dependency audit

Independently verify the cumulative M141-M145 production diff:

- M141, M142 and M144 production changes remain under `emissary-cli/src/i2pcontrol/**` except already accepted composition paths;
- M143 core changes are limited to exact neutral SAM streaming owners recorded in M061;
- M145 core changes are limited to exact neutral destination/session and SAM owners recorded in M061;
- no Proposal/I2PControl/TunnelManager/JSON-RPC vocabulary appears in neutral core declarations;
- no broad `crypto/`, `netdb/`, `i2np/` or other prefix exemption exists;
- no new direct dependency escaped the i2pcontrol feature boundary;
- Yosemite remains exact optional Y005 with no floating/path/global override.

Do not broaden M061/M062 merely to make tests pass.

## 9. Work packages

### WP1 — current-head inventory

Recompute matrix/residuals, identify the true last production-bearing commit, enumerate all post-M139 production files, and classify full-suite failures as stale-historical versus behavioral.

### WP2 — test authority repair

Refactor stale historical aggregate assertions so milestone-local facts remain preserved while current aggregate counts are owned by M153. Add a current-head M153 guard.

### WP3 — integrated runtime/security qualification

Run the control-plane, lifecycle, M140-M146 focused/adversarial suites and current production live-runtime tests. Add only qualification tests required to expose cross-feature regressions.

### WP4 — containment/dependency requalification

Mechanically compare post-M139/post-M145 production paths to M061/M062 and optional dependency activation.

### WP5 — authority/docs reconciliation

If clean:

- update M095 `current_production_head` to the actual accepted production head;
- mark M153 as current runtime/security qualification authority;
- demote M139 to historical current-head authority;
- update registry, residual/full-support roadmaps, implementation README and active support docs;
- leave M146 blocked and all 29 residual cells unchanged;
- do **not** register M147 automatically unless M154 (signature pre-registration audit) has separately closed.

### WP6 — closure

Write `plans/closure/i2pcontrol-proposal-170/153-closure.md` with exact commands/results, changed paths, matrix hash/counts, production-head determination, stale-test disposition table, unresolved findings and M154 readiness.

## 10. Required verification

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
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --test m153_post_m146_requalification --no-fail-fast
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Known repository-wide fmt/clippy drift may be recorded only if mechanically proven pre-existing and untouched. M153-touched files must be clean under the project's accepted formatter/toolchain.

## 11. Acceptance criteria

M153 closes complete only if:

- M095 mechanically remains `336/29/475` with exact 29-cell residual identity;
- no Proposal cell disposition changes;
- the actual production head is recorded correctly;
- the ordinary feature-enabled I2PControl test suite no longer fails solely because historical tests assert obsolete current aggregate counts/wording;
- M141-M145 runtime/security behavior composes on the final production source, including M145 no-std after `7cbd80a...`;
- M146 blocked behavior remains fail-closed and production-neutral;
- M127-M129 and lifecycle security regressions are green;
- M061/M062 exact containment/dependency isolation is truthful and not weakened;
- no medium/high correctness/security finding remains;
- M153 becomes the current runtime/security qualification authority;
- no production Rust/dependency/Yosemite change occurred.

## 12. Stop conditions

Stop and require a separate corrective implementation plan if:

- any current `apply` cell is inert/approximate;
- a behavioral regression is found in M127-M145;
- a production code/dependency change is needed to restore qualification;
- current production paths cannot be reconciled without a broad containment waiver;
- matrix counts/residual identities differ from `336/29/475` for reasons other than a separately accepted closure;
- full-suite failures cannot be explained as historical assertion drift.

## 13. External-interaction boundary

All external specification/reference access is read-only. M153 authorizes no upstream issue/PR/review/submission/contact or mutation. Repository writes remain internal to `eggstack/emissary`.

## 14. Closure evidence required

Record:

- implementation/qualification HEAD;
- exact last production-bearing HEAD and rationale;
- M095 hash/counts/residual identity;
- stale historical test disposition table;
- complete verification commands/results;
- M141-M146 composition/security evidence;
- M061/M062 cumulative path/dependency comparison;
- no-production-diff proof for M153;
- unresolved findings by severity;
- authority transition M139 -> M153;
- M154 readiness decision;
- external read-only attestation.
