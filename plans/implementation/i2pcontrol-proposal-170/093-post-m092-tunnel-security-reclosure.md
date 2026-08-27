# M093 — Post-M092 Tunnel Security Reclosure

Status: ready

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`.

Hard dependency:

- M092 `plans/implementation/i2pcontrol-proposal-170/092-m091-authorization-and-containment-corrective.md` must have an accepted `plans/closure/i2pcontrol-proposal-170/092-closure.md`.

Planning baseline: `944da7b887b6efbd46601e9fad1c853581f40b8e`.

Classification: independent security closure / no-production-change review.

## 1. Objective

Perform an independent current-head security/anonymity reclosure of all twelve Proposal 170 tunnel backends after M092 has removed M091's unauthorized Yosemite/core dependency expansion and restored the prior containment boundary.

M093 is evidence and closure work only. It MUST NOT make production changes. Any new high- or medium-severity production defect opens a new numbered corrective plan and keeps M093 blocked.

## 2. Readiness blocker

M092 has an accepted closure at `plans/closure/i2pcontrol-proposal-170/092-closure.md`. M093 is the dependency-ready next security reclosure and is now the active implementation handoff.

The reclosure must audit the actual corrected head, not the current M091-vendored head and not the older M089 pinned head. M093 is now executable against the M092-corrected head.

## 3. Review scope

Audit all twelve Proposal 170 tunnel types:

- `client`;
- `httpclient`;
- `ircclient`;
- `socks`;
- `socksirc`;
- `connectclient`;
- `streamrclient`;
- `server`;
- `httpserver`;
- `httpbidirserver`;
- `ircserver`;
- `streamrserver`.

The review must cover their shared runtime/admission/filter/session ownership as actually composed at the M092-corrected head.

## 4. Required authority checks

M093 must verify that the current repository state follows the authority order in `plans/003-planning-process.md`:

1. canonical specification/terminology;
2. accepted ADRs;
3. subsystem roadmaps;
4. implementation plans;
5. current code evidence.

Specifically verify:

- M090 remains correctly closed and present;
- M091 is no longer represented as valid current implementation authority after M092 rollback;
- M092 closure accurately records the rollback and containment restoration;
- M088 again truthfully owns the pre-accept/lower-layer residual limitation;
- no closure record claims an unapproved lower-layer defense remains active.

## 5. Security/anonymity review requirements

### 5.1 Server local-target confinement

Recheck that `httpserver`, inbound `httpbidirserver`, and `ircserver` normalize the accepted compatibility spellings to literal loopback socket addresses before connection and do not depend on resolver/NSS behavior.

Confirm no LAN/arbitrary clearnet target expansion was introduced.

### 5.2 Generic server lifetime and half-close

Verify M087 behavior remains intact:

- finite zero-progress inactivity bound;
- progress in either direction resets the deadline;
- one-sided EOF permits useful opposite-direction drain;
- no absolute maximum lifetime is imposed on an active stream;
- local target connect remains bounded.

### 5.3 IRC server lifecycle

Verify:

- bounded registration parsing;
- trusted peer-derived identity presentation;
- literal-loopback local target;
- five-second target connect;
- ten-minute progress-resetting inactivity behavior;
- corrected M090 half-close drain;
- raw post-registration bytes without newly invented application semantics.

### 5.4 HTTP server family

Recheck:

- accepted peer identity derives only from Yosemite/SAM;
- bounded/canonical Destination parsing and accounting identity;
- spoofable I2P/proxy identity headers are stripped/replaced;
- request framing is unambiguous/fail-closed;
- unsupported `Expect` requests fail before local target allocation;
- response fingerprint headers remain stripped;
- POST state/accounting remains bounded and keyed by canonical trusted identity;
- finite body/relay deadlines remain present;
- no speculative body byte cap or fairness redesign is added merely for closure.

### 5.5 Application admission

Verify the common post-accept `ServerAdmissionState` remains authoritative for:

- global concurrency;
- per-peer concurrency;
- configured minute/hour/day peer and aggregate rate windows;
- bounded peer-history/cardinality semantics;
- transactional admission/release;
- handler task ownership.

Explicitly state that this occurs after Yosemite `Session<style::Stream>::accept()` and does not prevent all lower-layer signed-SYN/streaming work.

### 5.6 Pre-accept residual

M093 must re-establish the exact corrected architecture after M092:

```text
remote signed streaming SYN
  -> Emissary lower-layer parse/signature/replay/stream work
  -> Yosemite Session<style::Stream>::accept()
  -> TrustedPeerIdentity
  -> ServerAdmissionState
  -> bounded application handler/local target
```

The absence of M091's pre-allocation stream-concurrency check is intentional under M092. Record the residual as a known availability/timing limitation, not as a direct clearnet identity leak.

Do not reopen a lower-layer implementation plan unless the maintainer separately authorizes a concrete dependency/core strategy.

### 5.7 Streamr

Verify the existing specialized model remains:

- loopback-only local endpoints;
- ten subscribers;
- 60-second expiry;
- 15-second refresh;
- one-byte control messages;
- 1200-byte application payload cap;
- bounded receive/fanout behavior.

Record that a Sybil adversary can monopolize the finite subscriber set. Treat that as the previously accepted reference-aligned availability limitation unless new concrete evidence shows a fork-specific high/medium defect.

### 5.8 `httpbidirserver`

Confirm the fork's separate unpublished outbound client session remains isolated from the public server Destination unless a separately authorized plan changes it. Do not adopt Java I2P's same-manager identity sharing merely for parity.

### 5.9 Persistent identity and diagnostics

Recheck:

- backend-owned persistent server keys;
- path confinement and symlink defenses;
- restrictive file mode where supported;
- no private Destination/key material in logs/errors;
- generation-local ephemeral runtime state.

## 6. Containment review

Audit the diff from M090 closure head through M092 corrected head.

Required conclusion:

- M090's production delta is retained;
- M091's `Cargo.toml`, `Cargo.lock`, `vendor/yosemite/**`, core, accepted-server lower-layer option, and self-authorizing containment delta are absent;
- M060/M061/M062 semantics again enforce the pre-M091 dependency/core boundary;
- only exact planning/closure bookkeeping for M092/M093 was added beyond the valid M090 state;
- no broad production path glob was introduced.

Any unexplained core/router/dependency change is a blocker.

## 7. Compatibility and protocol review

Verify no change to:

- Proposal 170 JSON-RPC methods/actions/statuses/field spelling;
- the twelve supported tunnel type names;
- startup-managed tunnel ownership;
- RouterInfo accepted 37/1/5 availability matrix;
- AddressBook/base-I2PControl unrelated limitations;
- public storage format;
- tunnel cryptographic algorithms, path selection, or transport algorithms.

Unsupported or underspecified runtime options must still fail before allocation rather than persist-and-ignore.

## 8. Failure, cancellation, restart, and contention review

For each server family and shared runtime owner, verify:

- admission leases release on handler completion/error;
- task groups are bounded and stop/restart does not leak generation state;
- no mutex/lock is held across network I/O, sleeps, or task joins;
- local target connect failures do not retain admission indefinitely;
- half-close and inactivity behavior match the documented family semantics;
- parser and peer-state failure paths remain bounded;
- session shutdown tears down owned tasks/state.

M093 does not need public-network stress or deanonymization testing. Use deterministic local/unit/integration evidence.

## 9. Required tests and verification

At minimum run:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-core
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m060_containment --test m061_containment --test m062_dependency_containment
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

Also run focused tests for:

- generic server inactivity/half-close;
- HTTP local target normalization and HTTP security filters;
- IRC registration/connect/inactivity/half-close;
- shared accepted-server admission/peer identity;
- Streamr local boundary/subscriber expiry/fanout;
- persistent server-key path/mode/symlink protections.

Record exact commands and counts in closure evidence.

Formatting-only failure caused by existing repository/nightly rustfmt configuration drift may be recorded, but M093 must not create formatter-only production churn.

## 10. Independent review rule

M093 must be reviewed against the actual M092-corrected head. Do not simply restate M089/M090/M092 closure claims.

For each security-critical invariant, cite current code/test evidence and distinguish:

- directly verified behavior;
- inherited historical evidence still applicable;
- accepted residual limitation;
- unavailable/unsupported semantic;
- new defect.

A code commit, passing compile, or prior closure assertion alone is insufficient.

## 11. Stop conditions

M093 MUST stop and open a new numbered corrective if it finds:

- a high- or medium-severity production security/anonymity defect;
- an unexplained M091 production/dependency artifact remaining after M092;
- a containment guard weakened beyond exact planning bookkeeping;
- a direct identity disclosure path;
- unbounded attacker-controlled server state/task growth within the reviewed application boundary;
- a Proposal 170 contract regression;
- a required fix that would change production code.

Low-severity/documentation findings may be corrected only if they do not alter production behavior and are explicitly recorded.

## 12. Closure evidence required

Create `plans/closure/i2pcontrol-proposal-170/093-closure.md` containing:

- reviewed commit SHA;
- dependency on accepted M092 closure;
- requirement-to-evidence matrix for all twelve tunnel types;
- focused M090/M092 regression evidence;
- application-admission and pre-accept ordering evidence;
- HTTP/IRC/Streamr security review;
- persistent identity/diagnostic review;
- exact containment/diff review;
- verification commands/outcomes;
- unresolved findings with severity;
- explicit accepted residuals, including lower-layer pre-accept work and Streamr Sybil monopolization;
- disposition: closed, corrective pass required, or blocked;
- internal-only/no-upstream attestation.

Only an accepted M093 closure may become current-head tunnel-security closure authority after M092.

## 13. Internal-only rule

All work is internal to `eggstack/emissary`.

External I2P/I2P+/Yosemite specifications and implementations are read-only reference evidence. No upstream issue, PR, review request, maintainer contact, submission, merge, patch series, or contribution preparation is authorized.