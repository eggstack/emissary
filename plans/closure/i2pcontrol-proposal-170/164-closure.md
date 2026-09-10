# M164 Closure — SAM Invalid-Command Secret-Redaction Corrective

Status: **closed as complete; zero Proposal promotions; M165 unblocked**

Date: `2026-09-10`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/164-sam-invalid-command-secret-redaction-corrective.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m152-blocked-state-corrective-roadmap.md`

Repository baseline reviewed: `9fe3fc8e` (M164 registered / dependency-ready)

Implementation and closure commit:

- This closure commit — one-file neutral SAM corrective, zero Proposal promotions.

Promotion budget: **zero Proposal cells**. M095 remains `336 apply / 29 blocked_primitive / 475 not_applicable`.

## 1. Executive finding

M164 is complete. Rejected, UTF-8 SAM command payloads are no longer included
in the invalid-command warning. The event records only the stable observation
identifier, accepted peer address, and command byte length. Invalid UTF-8
continues to log decoding metadata without raw bytes, and the existing stream
close behavior is unchanged.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| Rejected commands never log raw command text or bytes | `SamSocket` parse-failure branch and captured tracing test | pass | No `command` field is emitted; payload is omitted wholesale |
| Lookup-secret, PSK, and DH markers cannot enter warning fields | `invalid_command_log_omits_rejected_secret_bearing_payload` | pass | Unique markers for all three families are absent from captured events |
| Safe structural metadata remains available | Same captured tracing test | pass | Observation ID, peer, and exact command byte length are asserted |
| Invalid UTF-8 does not expose raw bytes | Existing invalid-input branch review and core regression suite | pass | Only `Utf8Error` metadata is logged; no byte payload field exists |
| Parser/session/connection behavior remains unchanged | Focused socket tests and complete `emissary-core` suite | pass | Rejected input still reaches the existing EOF/close behavior |
| Valid SAM command behavior remains unchanged | Complete `emissary-core` and I2PControl-enabled CLI suites | pass | HELLO, naming, session, streaming, and datagram coverage remains green |
| No ad-hoc secret-key redaction list is required | Source review | pass | Rejected content is omitted rather than tokenized or selectively redacted |
| Exact containment and zero-promotion budget | M062 registration/closure guard and changed-path review | pass | One production file; no dependency, manifest, Yosemite, or Proposal change |

## 3. Production implementation evidence

Exactly one production file changed:

1. `emissary-core/src/sam/socket.rs`

The test-only tracing capture is in the same authorized file. Planning/test
bookkeeping also repaired the pre-existing M163 registration guard omission in
the M062 historical allowlist; it does not widen the production budget.

No `emissary-util/**`, I2PControl production source, dependency, manifest,
lockfile, Yosemite, parser/session implementation, crypto, publication,
Proposal matrix, or runtime behavior changed.

## 4. Verification executed

### Commands run

```bash
rtk cargo test -p emissary-core invalid_command_log_omits_rejected_secret_bearing_payload -- --nocapture
rtk rustup run nightly rustfmt --check emissary-core/src/sam/socket.rs
rtk cargo test -p emissary-core
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m163_registration_guard
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix
rtk cargo check -p emissary-cli --no-default-features --features i2pcontrol
rtk cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
rtk cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- \
  -D warnings -A clippy::chunks-exact-to-as-chunks \
  -A clippy::field-reassign-with-default
```

### Results

- Focused M164 capture: pass, 1 test.
- Complete `emissary-core`: pass, 1,230 passed and 2 ignored.
- Complete I2PControl-enabled CLI suite: pass after repairing the inherited
  M163 guard allowlist; all library, binary, and integration suites are green.
- M062 containment/registration guard: pass.
- M163/M164 registration guard: pass.
- M095 matrix suite: pass; `336/29/475` unchanged.
- I2PControl feature `cargo check`: pass.
- Focused clippy command: pass with no M164 findings.
- Strict installed-Clippy and repository-wide nightly formatter checks retain
  unrelated pre-existing baseline failures; `socket.rs` itself passes nightly
  rustfmt, and the focused clippy invocation is clean with only those baseline
  lints excluded.

## 5. Invariant review

- The complete rejected SAM payload is treated as sensitive and never enters
  the warning event as a field or message value.
- No command-family inference or key-name allow/deny list is used.
- Observation ID, peer address, and byte length are structural metadata only.
- Valid SAM parsing and session retention are untouched.
- Invalid-command control flow remains unchanged: parser rejection is warned,
  then the socket continues through its existing read path; EOF still closes it.
- No Proposal capability was accepted, promoted, or demoted.

## 6. Security review

- Unique lookup-secret, PSK, and DH markers are absent from captured warning
  fields.
- No raw invalid UTF-8 bytes are logged; the existing `Utf8Error` contains only
  decoding-position metadata.
- No private/auth material is copied into a second diagnostic, metric, or
  persistence location.
- No parser relaxation, plaintext fallback, connection semantic change,
  dependency, Yosemite, or broad M061/M062 waiver was introduced.
- M146, M147/M148, M161, M162, and superseded M149-M151 dispositions remain
  unchanged.

## 7. Documentation and successor disposition

The M164 plan, M062 dependency record, implementation README, registry,
roadmap, AGENTS guidance, and the M163 registration guard now record M164 as
closed and M165 as the sole registered dependency-ready successor.

M165 is zero-production/zero-promotion and remains responsible for refreshing
M095 `current_production_head` to the actual last production-bearing M164
closure commit, then requalifying the whole surface. M095 is intentionally not
edited by M164.

## 8. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | M095 production-head metadata remains stale until final qualification | Intentionally outside M164; no support claim is affected | M165 refreshes the pointer and requalifies the corrected head |

No high or medium M164 finding remains. The temporary pre-closure M062
allowlist defect was corrected and is covered by the final containment tests.

## 9. Closure decision

M164 is **closed as complete**. Its exact one-file neutral core hardening is
implemented and verified, its zero Proposal-promotion budget is preserved, and
M165 is unblocked and registered as the next handoff.
