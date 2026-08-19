# I2PControl Proposal 170 Tunnel Security Hardening Roadmap

Status: corrective pass required; M083 next; M077-M079 blocked

Original planning baseline: `04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`.

Post-M076 corrective baseline: `1618de172e7a78a193fc1bb117af269f31174030`.

Current corrective baseline: `a35d2bc333ff0e8b9889cd133d8ef75a98faa049`.

Source runtime roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Canonical/internal authority:

- `plans/000-long-term-specification.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`;
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`;
- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`;
- M061 source-containment and M062/M063 dependency-containment authorities;
- M072 runtime reclosure and M073 option-truthfulness history.

Pinned external contract:

- I2P Proposal 170, `I2PControl Expansion`, Open revision created/updated `2026-05-20`.

External I2P/I2P+/Yosemite sources remain read-only behavioral/security evidence. No upstream mutation, review request, issue/PR submission, contribution preparation, merge request, or maintainer contact is authorized.

## 1. Purpose

M066-M071 made the previously deferred Proposal 170 tunnel families operational. M074-M076 then added server admission, generic accepted-stream handling, and HTTP anonymity/POST hardening. The first independent security review inserted M080-M082, which successfully corrected the original aggregate-rejection state leak, 64-bit peer keys, stale expiry-queue growth, generic-server `leaseSetEncType` regression, brittle HTTP peer-length assumption, and `Expect: 100-continue` wait cycle.

A second independent review of head `a35d2bc` found that the shared accepted-server boundary still has four narrow correctness gaps:

- minute-scale historical peer state is conflated with no-history cleanup, so an unlimited aggregate policy can remain accepted where retained peer cardinality is unrepresentable;
- the capacity solver uses aggregate-field precedence rather than the tightest bound implied by all enabled aggregate windows;
- expired active peers can lose the expiry-index representation claimed by M080;
- trusted Destination parsing does not require zero unconsumed bytes and preserves attacker-selected textual representation downstream.

These are localized defects in otherwise-correct seams. M083 corrects them before the original IRC/Streamr/final-reclosure sequence resumes.

The workstream remains bounded to Proposal 170 tunnel security/correctness. It does not add tunnel types, redesign router protocols, add HTTP features, implement arbitrary I2CP pass-through, or broaden startup-service ownership.

## 2. Security/anonymity model

The main concern is controllable remote state: a peer should not be able to create a stable saturation, long-lived occupancy, metadata fingerprint, or textual-identity alias that becomes externally correlatable or denies unrelated peers.

Target defenses:

- rejected traffic never poisons durable in-memory accounting;
- attacker-influenced state is finite and its capacity proof matches actual rate-window semantics;
- historical peer state exists only while configured peer-rate semantics require it;
- no-history inactive peers do not accumulate merely because of cleanup implementation details;
- trusted peer identity is exactly one supported Destination and downstream text is canonical;
- accepted-server protocol handlers receive a single shared trusted identity model;
- HTTP/IRC/Streamr resource occupancy remains bounded;
- backend/provider/cache/trace fingerprints remain suppressed;
- runtime configuration remains apply-or-reject truthful.

Random jitter, fixed overload delays, artificial response padding, and public-network deanonymization experiments remain non-goals.

## 3. Retained accepted results

### 3.1 M080 mechanisms to retain

M080 correctly established:

- transactional admission checks before peer/counter/expiry mutation;
- 32-byte cryptographic Destination ID accounting;
- bounded peer state under a documented hard memory ceiling;
- a one-authoritative-entry expiry-index design direction using `BTreeMap`;
- retention based on enabled peer windows rather than unconditional day retention;
- Tokio `test-util` moved back to dev-only activation;
- structurally parsed trusted identity before handler/local-target work.

M083 repairs semantic edges without reverting these mechanisms.

### 3.2 M081 is accepted

Generic control-plane `server` remains accepted-stream/raw-relay. Validated `leaseSetEncType` reaches Yosemite `SessionOptions::lease_set_enc_type`; other accepted-server families explicitly do not gain that option.

M083 does not reopen M081.

### 3.3 M082 mechanisms to retain

M082 correctly:

- routes HTTP through the common `TrustedPeerIdentity` boundary;
- rejects every `Expect` header with fixed 417 semantics before local target allocation;
- keys POST throttling on the 32-byte Destination ID;
- retains M076 request/proxy-spoof stripping, response fingerprint suppression, and framing checks.

M083 only corrects exact/canonical full-Destination semantics inherited from the shared identity boundary.

## 4. Post-M082 findings

### 4.1 Minute/no-history representability — MEDIUM

`MINUTE` and `SHORT_RETENTION` are both 60 seconds, while the M080 capacity check is gated by `retention > SHORT_RETENTION`. Minute-only peer-rate history therefore skips the check that rejects fully unlimited aggregate arrival when retained cardinality cannot be bounded by policy.

Additionally, when every per-peer rate is unlimited, inactive records currently remain for a short retention despite no historical peer-rate semantics being needed. Under unlimited aggregate churn, that creates avoidable table pressure.

M083 separates peer-history requirement from cleanup duration. No-history inactive peers are reclaimed promptly; minute/hour/day historical policies are capacity-checked regardless of duration equality.

### 4.2 Aggregate bound selection — MEDIUM-LOW

The current helper selects the first non-zero aggregate field rather than evaluating all enabled minute/hour/day fields. This can falsely reject a representable policy when a tighter hour/day bound exists.

M083 computes a safe retained-event upper bound for each enabled aggregate window, includes fixed-window boundary overlap, and takes the minimum because the runtime enforces all enabled aggregate fields conjunctively.

### 4.3 Active-peer expiry-index consistency — LOW-MEDIUM

M080's `reap` path can remove an active peer's expiry-index entry while leaving the peer record active. M083 must make the intended invariant explicit and preserve it across acquire/reap/drop.

### 4.4 Trusted Destination exactness/canonical text — LOW-MEDIUM

The shared helper uses the core convenience `Destination::parse`, which does not require `parse_frame` remainder to be empty. A valid Destination plus trailing bytes can therefore produce a trusted 32-byte ID while preserving the original textual representation for HTTP metadata/access matching.

M083 requires exactly one parsed Destination with no remainder and derives downstream full-Destination text from canonical Base64 encoding of the parsed serialized bytes. No core parser change is authorized.

## 5. Security invariants

All M083/M077-M079 work MUST preserve:

- exact Proposal 170 wire fields/actions/types; no security-only wire extensions;
- authenticated remote identity from SAM/Yosemite only;
- exactly-one-Destination trusted identity validation with canonical downstream text and a fixed 32-byte cryptographic accounting ID;
- policy/admission decisions before local target work where technically possible;
- side-effect-free denial apart from bounded reclamation of already-expired state;
- bounded tasks, peer state, expiry indexes, counters, buffers, and shutdown waits;
- no active/throttled state evicted merely to admit a new attacker-controlled identity;
- peer-rate history retained only as long as enabled semantics require it;
- capacity math that cannot understate actual fixed-window accepted cardinality;
- no lock across network I/O, sleeps, target connect, handler execution, joins, or shutdown waits;
- monotonic time for rate/idle state;
- no private destination material/full raw config values in diagnostics;
- no random/fixed timing-jitter defenses;
- no local DNS/LAN routing expansion;
- no new `emissary-core/**` production path;
- no startup-service ownership refactor;
- no hosted CI/fuzz/soak/release machinery for this workstream;
- no upstream write/review/submission activity.

## 6. Target accepted-server architecture

```text
SAM/Yosemite accepted stream
    -> I2PControl TrustedPeerIdentity
       - bounded input text
       - decoded payload is exactly one supported Destination
       - canonical full-Destination text for handlers
       - canonical fixed-size Destination ID for accounting
    -> I2PControl ServerAdmissionState
       - transactional denial
       - global + peer concurrent limits
       - peer + aggregate minute/hour/day counters
       - explicit peer-history horizon
       - capacity from tightest safe enabled aggregate bound
       - fixed-window boundary overlap included
       - bounded expiry index with a documented active/inactive invariant
    -> protocol-specific pre-local validation
    -> fixed administrator-selected safe local target policy
    -> bounded protocol relay/filter
```

Generic server remains raw after admission. HTTP/IRC remain protocol-filter owners. Streamr remains its own bounded datagram family.

## 7. Capacity model required by M083

Do not use duration comparison to infer whether historical peer state exists.

For historical minute/hour/day peer-rate state:

1. identify the longest enabled peer-history horizon;
2. for each enabled aggregate `(limit, window)`, derive a safe maximum accepted count over that horizon under the current fixed-window counter semantics;
3. account for a retained interval crossing aggregate-window boundaries — `limit * (ceil(history/window) + 1)` is an acceptable conservative starting point, or use a tighter proven equivalent;
4. take the smallest safe bound across all enabled aggregate fields;
5. add a documented finite concurrency/edge margin;
6. reject when every aggregate field is unlimited or the resulting requirement exceeds the hard peer-state ceiling.

For no-history policy, active peer records are needed only for concurrency. Once a peer has no active lease, no historical peer record should remain solely due to an arbitrary short cleanup retention.

Do not silently weaken configured rate values or evict live historical state.

## 8. Corrective dependency graph

```text
current head a35d2bc
      |
      v
M083 admission capacity / expiry / exact trusted identity
      |
      v
M077 IRC lifetime/exhaustion hardening
      |
      v
M078 Streamr local-boundary hardening
      |
      v
M079 integrated tunnel-security reclosure
```

Historical corrective sequence retained for traceability:

```text
M080 admission transactionality/cardinality
  -> M081 generic server LeaseSet truthfulness
  -> M082 HTTP peer identity / Expect / POST key
  -> M083 post-M082 shared-boundary corrective
```

Dependency classification:

- post-M082 review -> M083: corrective hard gate;
- M083 -> M077: hard gate because IRC consumes the shared admission/trusted-peer boundary;
- M077 -> M078: registry sequencing;
- M083 + M077 + M078 -> M079: hard final closure dependencies.

## 9. Milestone summary

### M074 — Shared admission hardening

Historical architecture retained. Original defects were corrected by M080; M083 closes remaining capacity-state semantics discovered after M080 closure.

### M075 — Generic accepted-stream migration

Closed. M081 repaired its option-truthfulness regression.

### M076 — HTTP anonymity/POST hardening

Closed with corrective history. M082 repaired its direct identity-length/`Expect`/POST-key defects; M083 revalidates exact canonical full-Destination semantics inherited from the shared identity layer.

### M080 — Admission transactionality/cardinality corrective

Historical closure accepted for its pinned implementation evidence, but current security disposition is `corrective pass required` until M083 closes minute/no-history capacity semantics, true aggregate-bound derivation, and active-peer expiry consistency.

### M081 — Generic server LeaseSet option truthfulness corrective

Closed and not reopened by M083.

### M082 — HTTP peer identity and Expect-framing corrective

Direct `Expect` and POST-key fixes are retained. Current trusted-Destination exactness remains corrective through M083.

### M083 — Admission capacity semantics and trusted Destination exactness corrective

Sole ready handoff. Correct the shared boundary without expanding architecture.

### M077 — IRC lifetime/exhaustion hardening

Blocked until M083. Then add a 10-minute activity-resetting post-registration inactivity deadline and bounded target connect while preserving registration filtering/raw post-registration semantics.

Status: closed; closure: `plans/closure/i2pcontrol-proposal-170/077-closure.md`.

### M078 — Streamr local-boundary hardening

After M077, make local UDP producer/client targets loopback-only, align subscriber maximum to 10, and preserve existing expiry/refresh/payload/task bounds.

Status: closed; closure: `plans/closure/i2pcontrol-proposal-170/078-closure.md`.

### M079 — Integrated tunnel-security reclosure

Independently re-audit the actual final head. Rebuild option-capability truthfulness and threat-model resource/timing/fingerprint behavior, verify containment/lifecycle, and refuse closure with any high/medium finding.

## 10. M079 final evidence requirements

M079 must explicitly prove at final head:

- aggregate-/peer-/global-denied attempts cannot grow accounting state;
- no-history inactive peer churn cannot create table-sized occupancy;
- minute/hour/day historical peer policies are representable or rejected before allocation;
- capacity uses the tightest safe bound across enabled aggregate windows and includes boundary overlap;
- every admission/POST auxiliary expiry structure is hard bounded with a coherent active/inactive invariant;
- canonical 32-byte Destination hashes key security accounting;
- trusted peer input contains exactly one supported Destination and downstream text is canonical;
- generic `leaseSetEncType` is applied or rejected, never ignored;
- `Expect` cannot create a body/backend wait cycle;
- one peer cannot monopolize accepted-server capacity;
- HTTP fingerprint/proxy identity stripping remains effective;
- IRC idle occupancy is finite and activity-resetting;
- Streamr local UDP exposure is loopback-only and fanout bounded;
- lifecycle/stop/restart clears ephemeral security state and cannot cross generations;
- M061/M062/M063 containment remains intact;
- no current high/medium security/anonymity/correctness finding remains.

## 11. Verification discipline

Use focused deterministic tests, structurally valid I2P Destination fixtures plus trailing-byte negatives, fake/local SAM endpoints, local TCP/UDP capture services, Tokio paused-time tests, package-scoped `cargo test`/`cargo check`, M061/M062/M063 containment suites, Clippy, scoped nightly rustfmt for touched files, and `git diff --check`.

Do not create public-network certification/deanonymization tests, load-test farms, hosted CI jobs, generalized fuzz infrastructure, benchmark gates, or release/upstream machinery.

## 12. Stop conditions

Stop the affected milestone and create separate architecture/corrective planning if:

- exact trusted identity requires a new `emissary-core/**` or SAM API instead of composing existing parser primitives;
- exact configured rate semantics cannot be represented within the existing defensible hard memory ceiling and cannot be rejected before allocation;
- a correction requires silently weakening configured Proposal 170 rates;
- IRC inactivity cannot be implemented without parsing/reframing post-registration traffic;
- Streamr compatibility is proven to require unauthenticated non-loopback exposure;
- a new I2PControl-only dependency violates M062/M063 ownership;
- M079 finds any remaining high/medium issue.

## 13. Final closure rule

The tunnel-security workstream remains `corrective pass required` until M083, M077, and M078 are independently closed and M079 accepts the actual final head.

Historical M073-M082 closure evidence remains useful for what the pinned commits demonstrated, but it does not override later independent findings. M079 is the final authority for closing this line of work.
