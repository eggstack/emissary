# M165 Closure — Post-Corrective Current-Head Requalification

Status: **closed as complete; safe-partial current-head authority; no successor**

Date: `2026-09-10`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/165-post-corrective-current-head-requalification.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m152-blocked-state-corrective-roadmap.md`

Hard dependencies:

- M163 closed complete at `9fe3fc8e`;
- M164 closed complete at `0dbaa6b1`.

Reviewed production head: `0dbaa6b1082762f5f2c42b47dcb22c1002ea4bf9`

Promotion budget: **zero Proposal cells**.

## 1. Executive finding

M165 is complete. The post-M163/M164 repository head is requalified as the
current safe-partial Proposal-170 authority. M095 now points to the actual
last production-bearing commit, `0dbaa6b1082762f5f2c42b47dcb22c1002ea4bf9`.
The matrix remains exactly `336 apply / 29 blocked_primitive / 475
not_applicable` across 840 cells, and no cell disposition changed.

M163's fail-before-durable-effect and retained-generation scrub behavior and
M164's whole-payload SAM rejection logging hardening remain intact. No high or
medium correctness or security defect remains. No dependency-ready successor
is unblocked by this qualification.

## 2. Authority and matrix reconciliation

The last production-bearing commit after M164 is the M164 implementation head:

- `0dbaa6b1082762f5f2c42b47dcb22c1002ea4bf9` — `fix(sam): redact invalid command payloads`.

M165 changed only planning, qualification, guard, and authority metadata. No
production source, manifest, lockfile, Yosemite, frontend, workflow, or
runtime behavior changed during qualification.

M095 was mechanically recomputed:

- total: `840`;
- apply: `336`;
- blocked_primitive: `29`;
- not_applicable: `475`.

The exact blocked identities remain:

- `SigType` ×10;
- `EncryptLeaseSet` ×5;
- `OptionalLookup` ×5;
- `LeaseSetClientAuths` ×5;
- `UseOutproxyPlugin` ×4.

The only M095 content change is the `current_production_head` metadata line.
M095, M105, and the historical matrix guards agree with the recomputation.

## 3. Requirement-to-evidence matrix

| Qualification area | Evidence | Result |
|---|---|---|
| M061 exact containment and Proposal-vocabulary separation | M062 containment suite; exact M061 owner lists; no new production path or broad waiver | pass |
| M062 dependency containment and registration closure | M062 dependency suite plus updated closed-registration guard; empty current registration after M165 | pass |
| M095 matrix and residual identities | M095 suite plus independent TOML recomputation; `336/29/475` and `10/5/5/5/4` residual split | pass |
| M105 residual option audit | M105 suite; all 840 option/family cells remain mechanically consistent | pass |
| M127 finite token lifetime | Full I2PControl-enabled CLI suite and M127 guards | pass |
| M128 bounded JSON-RPC admission | Full I2PControl-enabled CLI suite and M128 guards | pass |
| M129 fail-closed management TLS | Full I2PControl-enabled CLI suite and M129 guards | pass |
| M135 ownership, live quantity and LeaseSet truthfulness | Full I2PControl-enabled CLI suite and M135 composition evidence | pass |
| Reduce/Close/IdlePolicy/NewDest composition | Full I2PControl-enabled CLI suite and lifecycle guards | pass |
| M141-M145 behavior | M141, M142, M143, M144 and M145 suites; core no-std check | pass |
| M146 provider blocking | M146 blocked-provider suite; no direct-clearnet or dummy-provider fallback | pass |
| M147/M148 and M154 SigType disposition | M095/M105/source guards; all ten cells remain blocked | pass |
| M156-M160 neutral LeaseSet primitives | Complete core suite, no-std check, source/containment guards; no Proposal promotion | pass |
| M161 legacy AES/LS1 | M161 outcome-B closure and current matrix guards; all five cells remain blocked | pass |
| M162 typed/redacted integration | Complete CLI suite, M062 guards, and M162 closure evidence; all fifteen cells remain blocked | pass |
| M163 durable-effect and history correction | M163 adversarial tests in the complete CLI suite: individual/combined fields, revision preservation, two clean fallbacks, purge failure/restart behavior, retention scrubbing and unrelated-state preservation | pass |
| M164 SAM log hardening | Focused capture plus complete core suite; whole rejected payload omitted, lookup/PSK/DH markers absent, structural metadata retained | pass |

Qualification distinguishes runtime/behavioral support from parser,
serializer, and persistence reachability. No inert storage or neutral
infrastructure is counted as Proposal support.

## 4. Verification executed

```text
rtk cargo test -p emissary-core
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol
rtk cargo check -p emissary-core --no-default-features --features no_std
rtk cargo check -p emissary-cli --no-default-features --features i2pcontrol
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol \
  --test m061_containment --test m062_dependency_containment \
  --test m095_full_support_matrix --test m105_residual_option_audit \
  --test m126_requalification --test m127_token_lifetime \
  --test m128_jsonrpc_batch --test m129_nonloopback_tls \
  --test m163_registration_guard
rtk cargo test -p emissary-core \
  invalid_command_log_omits_rejected_secret_bearing_payload -- --nocapture
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol \
  --test m163_registration_guard --test m095_full_support_matrix \
  --test m105_residual_option_audit --test m153_post_m146_requalification
rtk cargo clippy -p emissary-cli --no-default-features --features i2pcontrol \
  --all-targets -- -D warnings \
  -A clippy::chunks-exact-to-as-chunks \
  -A clippy::field-reassign-with-default
rtk cargo fmt --all -- --check
```

Results:

- `emissary-core`: **1,230 passed, 2 ignored**;
- I2PControl-enabled CLI: **2,306 passed across 41 suites**;
- focused containment/requalification guards: **36 passed across 5 suites**;
- focused post-corrective guard set: **10 passed across 4 suites**;
- M164 captured invalid-command test: **1 passed**;
- core no-std and I2PControl feature checks: **pass**;
- focused Clippy command: **pass**;
- strict installed-Clippy command: baseline failure only at the unrelated
  `chunks_exact` lint in `emissary-cli/src/i2pcontrol/backends/filters/proxy.rs`;
- stable workspace formatter check: baseline drift in unrelated pre-existing
  files; no M165 production formatting change was made.

## 5. Adversarial and security evidence

The M163 tests prove that each blocked field and their combinations are
rejected before Create/Edit store mutation, edits preserve the prior revision,
legacy contaminated generations enter the pending scrub, two clean fallback
generations are published before purge, retained generation files contain no
blocked names or secret bytes, interruption/failure prevents StartOnLoad and
resumes on the next load, and unrelated definitions/options survive.

The M164 capture proves that rejected UTF-8 command content and unique lookup,
PSK and DH markers are absent from warning fields while observation ID, peer,
and byte length remain available. Invalid UTF-8 continues to log only safe
decoding metadata. The implementation omits rejected payloads wholesale and
does not use a secret-key allow/deny list.

The final audit found no blocked option with durable or runtime effect, no
plaintext/unsecreted/unauthenticated fallback, no direct-clearnet outproxy
fallback, no broad M061/M062 waiver, no secret-bearing response/debug/log
surface, and no second LeaseSet publication or rollover owner.

## 6. Future-plan and dependency disposition

No future plan can be unblocked by M165:

- M146 remains closed blocked for `UseOutproxyPlugin`;
- M147 is closed blocked under M154 disposition C;
- M148 remains deferred/unregistered behind M147;
- M149-M151 remain superseded/unregistered;
- M161 remains outcome B and all fifteen M162 LeaseSet-security cells remain
  blocked;
- M159/M160 remain neutral infrastructure only.

The post-M152 corrective roadmap is closed. The post-M154 LeaseSet-security
roadmap is historical/superseded. The broader full-support roadmap remains
active/partial because the 29 accepted blockers remain. The registry and M062
now record no dependency-ready implementation handoff.

## 7. Closure decision

M165 is **closed as the safe-partial current-head qualification authority**.
M095 remains `336/29/475`, no Proposal cell was promoted or demoted, and no
registered successor is required. `terminal` continues to mean only that no
dependency-ready safe implementation plan exists under the accepted
architecture/policy; it does not convert any blocked cell into support.
