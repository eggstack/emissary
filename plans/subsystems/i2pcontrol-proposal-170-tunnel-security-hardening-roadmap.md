# I2PControl Proposal 170 Tunnel Security Hardening Roadmap

Status: corrective pass required; M080 next; M081-M082 and M077-M079 blocked

Original planning baseline: `04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`.

Current corrective baseline: `1618de172e7a78a193fc1bb117af269f31174030`.

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

Read-only reference snapshots used by this workstream:

- Java I2P `i2p/i2p.i2p`;
- I2P+ `I2PPlus/i2pplus`;
- the Yosemite version pinned by this repository.

External sources are behavioral/security evidence only. No upstream mutation, review request, submission, contribution preparation, or maintainer contact is authorized.

## 1. Purpose

M066-M071 made the ten previously deferred Proposal 170 tunnel families operational. M074-M076 then added substantial server-side hardening: peer-aware admission, generic accepted-stream handling, HTTP response-fingerprint suppression, proxy-identity stripping, and churn-safe POST accounting.

An independent review of current head `1618de1` found that this line of work is not yet security-closed. The architecture is directionally correct and remains well-contained, but several implementation invariants were either missed during closure or regressed during later migration.

This roadmap therefore inserts three narrow corrective milestones before the previously planned IRC/Streamr completion sequence:

- M080 — shared server admission transactionality/cardinality;
- M081 — generic server `leaseSetEncType` option truthfulness;
- M082 — HTTP valid-Destination/`Expect`/peer-key correctness.

After those close, the original M077 -> M078 -> M079 sequence resumes.

The workstream remains bounded to Proposal 170 tunnel security/correctness. It does not add tunnel types, redesign router protocols, introduce a general WAF, implement arbitrary I2CP pass-through, or broaden startup-service ownership.

## 2. Security/anonymity model

The primary concern is not that one timing observable directly reveals the host IP. The concern is that a remote I2P peer can deliberately create a stable condition — saturation, long-lived occupancy, backend/provider metadata, or injected/local traffic — and correlate that condition with a candidate service or host outside I2P.

The target defenses are therefore:

- prevent one peer from monopolizing accepted-server capacity;
- prevent rejected traffic from poisoning persistent in-memory accounting state;
- keep every attacker-influenced state structure hard bounded;
- make bounded capacity coherent with configured rate windows rather than creating an unnecessarily small long-lived choke point;
- prevent indefinite protocol-idle occupancy where the reference implementation has an established idle bound;
- strip backend clock/provider/cache/trace fingerprints;
- validate trusted peer identity structurally rather than using brittle magic lengths;
- avoid client/server protocol wait cycles such as unsupported `Expect: 100-continue`;
- preserve truthful apply-or-reject semantics for runtime configuration.

Random jitter, fixed overload delays, artificial response padding, and public-network deanonymization experiments remain non-goals.

## 3. Reference findings retained from the original hardening roadmap

### 3.1 Java I2P server admission is layered

Reference server behavior includes finite global concurrency plus peer-keyed and aggregate connection throttles. Reference-scale defaults used by this workstream are 30 global concurrent streams, 30/80/200 per peer per minute/hour/day, and 50/0/0 aggregate per minute/hour/day.

Security conclusion: a global semaphore alone is not sufficient; authenticated peer identity and aggregate rate state must participate before application/local-target work.

### 3.2 Java/I2P+ HTTP filtering treats metadata as an anonymity boundary

Java I2P removes at least `Date`, `Server`, `X-Powered-By`, `X-Runtime`, and proxy headers. I2P+ additionally strips provider/cache/trace metadata such as `Via`, cache headers, cloud trace values, and hosting identifiers.

Security conclusion: the M076 fingerprint denylist remains required and is not reopened by M082 except for regression verification.

### 3.3 Java IRC bounds registration and later idle lifetime

Reference IRC server behavior bounds registration and then applies a 10-minute read/inactivity timeout.

Security conclusion: M077 remains required after the corrective prerequisites close.

### 3.4 Streamr remains intentionally small

Reference Streamr behavior uses a finite subscriber set with 60-second expiry. Emissary's M071 implementation is already bounded but still permits non-loopback local UDP configuration.

Security conclusion: M078 remains required after M077.

## 4. Independent post-M076 findings

### 4.1 M074: aggregate-rejected new peers can poison admission state — HIGH

Current `ServerAdmissionState::try_acquire` may insert a new peer before aggregate-rate eligibility is known. If the aggregate check rejects the attempt, the new zero-active peer receives no successful counter record and no expiry registration. Since `reap()` only visits queued expirations, those records can persist for the lifetime of the runtime generation.

A remote attacker can first exhaust aggregate rate and then submit fresh authenticated Destinations to fill the peer table, denying all unseen peers until restart.

### 4.2 M074: auxiliary expiry state is not independently bounded — MEDIUM-HIGH

The primary peer map is bounded, but append-only expiry entries may accumulate independently. Security closure requires every attacker-influenced collection to be bounded, not only the main map.

### 4.3 M074: 4096 fixed peers is incoherent with long retained windows — MEDIUM-HIGH

Default peer-day accounting retains identities substantially longer than the time needed for aggregate-admissible fresh identities to fill a 4096-entry table. Fail-closed capacity protects memory but creates a predictable deny-new-peer condition well before retained state naturally expires.

The corrective target is not unbounded state. Capacity must instead be derived from the maximum distinct arrivals permitted by the configured aggregate policy over the longest enabled peer window, subject to a documented hard memory ceiling; unsafe configurations fail before allocation.

### 4.4 M074/M076: accounting keys use 64-bit `DefaultHasher` — LOW-MEDIUM

Authenticated I2P Destinations have a canonical cryptographic Destination ID/hash. Admission and HTTP POST accounting should use that fixed identity instead of an unspecified 64-bit general-purpose hash.

### 4.5 M075 regressed generic-server `leaseSetEncType` truthfulness — MEDIUM

M073 historically closed while generic server accepted `leaseSetEncType` and mapped it into the old Yosemite session configuration. M075 migrated control-plane generic server to accepted streams but did not carry the option into `AcceptedServerRuntimeConfig`/`SessionOptions`, while the backend still accepts it.

M081 must apply it in accepted-stream session setup or reject it before allocation. Restoring control-plane `STREAM FORWARD` is prohibited.

### 4.6 M076's 524-character peer bound rejects valid current Destinations — MEDIUM

The HTTP filter's bound was derived from a legacy-sized Destination representation. Current I2P key-certificate/signature forms can be larger. The repository already has structural full-Destination parsing and canonical ID derivation.

M082 replaces the magic validity ceiling with structural validation plus a bound derived from all currently supported Destination forms.

### 4.7 M076 allows unsupported `Expect: 100-continue` wait cycles — MEDIUM

The HTTP handler forwards request headers, then waits for the remote body before reading the local backend response. A client using `Expect: 100-continue` may wait for an interim response while the local backend has already emitted it, causing both sides to wait until body timeout.

M082 rejects all `Expect` requests before local target allocation instead of adding a broader informational-response state machine.

## 5. Security invariants

All remaining M080-M082/M077-M079 work MUST preserve:

- exact Proposal 170 wire fields/actions/types; no security-only wire extensions;
- authenticated remote identity from SAM/Yosemite only;
- structural trusted-Destination validation and canonical cryptographic identity;
- policy/admission decisions before local target work where technically possible;
- side-effect-free denial apart from bounded reclamation of already-expired state;
- bounded tasks, peer state, expiry indexes, counters, buffers, and shutdown waits;
- no active/throttled state evicted merely to admit a new attacker-controlled identity;
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
    -> structurally validated TrustedPeerIdentity
       - bounded canonical Destination text if protocol handler needs it
       - canonical fixed-size Destination ID/hash for accounting
    -> I2PControl-owned ServerAdmissionState
       - transactional denial
       - global + peer concurrent limits
       - peer + aggregate minute/hour/day counters
       - capacity derived from retained semantics within hard memory ceiling
       - bounded one-authoritative-expiry-per-peer index
    -> protocol-specific pre-local validation
    -> fixed administrator-selected loopback target
    -> bounded protocol relay/filter
```

Generic server stays raw after admission. HTTP/IRC remain protocol-filter owners. Streamr remains its own bounded datagram family.

## 7. Option truthfulness policy

Every runtime-relevant field has exactly one disposition:

- applied and verified in runtime/session configuration;
- invalid/irrelevant for that tunnel type and rejected;
- recognized but unsupported and rejected before allocation.

Persist-and-ignore is forbidden.

M081 specifically re-audits generic server `leaseSetEncType`. `PerClientPeriod`, `TotalPeriod`, `TotalBanTime`, `FilterFilePath`, `UniqueLocalAddressPerClient`, and `MultiHoming` remain fail-before-allocation unless a separately authoritative implementation plan establishes exact supported semantics.

No arbitrary I2CP/custom pass-through is introduced.

## 8. Corrective dependency graph

```text
current head 1618de1
      |
      v
M080 admission transactionality/cardinality
      |
      v
M081 generic server LeaseSet truthfulness
      |
      v
M082 HTTP peer identity / Expect corrective
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

Dependency classification:

- current review -> M080: corrective hard gate;
- M080 -> M081: registry sequencing; M081 also must preserve corrected admission behavior;
- M080 -> M082: hard interface dependency for canonical trusted-peer identity/accounting representation;
- M081 -> M082: registry sequencing to preserve one ready handoff;
- M080-M082 -> M077: hard corrective gate because M077 consumes the M074 accepted-server boundary and final security work must not proceed on invalidated prerequisites;
- M077 -> M078: registry sequencing from the original roadmap;
- M080-M082 + M077-M078 -> M079: hard final closure dependencies.

## 9. Milestone summary

### M074 — historical shared admission hardening

Status: corrective pass required at current head. M080 owns the discovered defects. Retain its reference defaults, per-peer fairness, common accepted-stream integration, and RAII lease design unless M080 proves a narrower correction is impossible.

### M075 — historical generic accepted-stream migration

Status: corrective pass required for current option-truthfulness invariant. Retain accepted-stream raw relay and loopback target. M081 owns `leaseSetEncType` apply-or-reject repair.

### M076 — historical HTTP anonymity/POST hardening

Status: corrective pass required. Retain fingerprint/proxy stripping, framing checks, and fail-closed POST table behavior. M082 owns peer identity, Expect, and cryptographic throttle key corrections.

### M080 — Server admission transactionality/cardinality corrective

Make all denial paths side-effect free; prevent aggregate-rejection peer leaks; bound expiry/index state; use canonical Destination IDs; make capacity/retention coherent with configured windows and hard memory budget; recheck Tokio test-util containment.

### M081 — Generic server LeaseSet option truthfulness corrective

Apply `leaseSetEncType` to accepted-stream Yosemite session configuration or reject before allocation. Do not restore `STREAM FORWARD` or add adjacent I2CP features.

### M082 — HTTP peer identity and Expect-framing corrective

Use structurally valid current Destination semantics instead of a legacy magic bound; reject unsupported `Expect` before local connect; use canonical Destination IDs in POST accounting; retain M076 anonymity/filter/framing work.

### M077 — IRC lifetime/exhaustion hardening

After M080-M082 close, add a 10-minute activity-resetting post-registration inactivity deadline and bounded target connect while preserving M066 registration filtering/raw post-registration semantics.

### M078 — Streamr local-boundary hardening

After M077, make local UDP producer/client targets loopback-only, align subscriber maximum to 10, and preserve existing expiry/refresh/payload/task bounds.

### M079 — Integrated tunnel-security reclosure

Independently re-audit the final actual head. Rebuild option-capability truthfulness, threat-model resource/timing/fingerprint behavior, verify containment/lifecycle, and refuse closure with any high/medium finding.

## 10. M079 final evidence requirements

M079 must explicitly prove at final head:

- aggregate-rejected/new-peer attempts cannot grow peer accounting;
- every admission/POST auxiliary expiry structure is hard bounded;
- default/configured retained-rate state and memory capacity are coherent;
- canonical Destination hashes key security accounting;
- generic `leaseSetEncType` is applied or rejected, never ignored;
- valid currently supported large Destinations pass trusted identity validation;
- malformed Destinations fail before local work;
- `Expect` cannot create a body/backend wait cycle;
- one peer cannot monopolize accepted-server capacity;
- HTTP fingerprint/proxy identity stripping remains effective;
- IRC idle occupancy is finite and activity-resetting;
- Streamr local UDP exposure is loopback-only and fanout bounded;
- lifecycle/stop/restart clears ephemeral security state and cannot cross generations;
- M061/M062/M063 containment remains intact;
- no current high/medium security/anonymity/correctness finding remains.

## 11. Verification discipline

Use focused deterministic tests, structurally valid I2P Destination fixtures, fake/local SAM endpoints, local TCP/UDP capture services, Tokio paused-time tests, package-scoped `cargo test`/`cargo check`, M061/M062/M063 containment suites, Clippy, scoped nightly rustfmt for touched files, and `git diff --check`.

Do not create public-network certification, deanonymization experiments, load-test farms, hosted CI jobs, generalized fuzz infrastructure, benchmark gates, or release/upstream machinery.

## 12. Stop conditions

Stop the affected milestone and create separate architecture/corrective planning if:

- canonical peer identity requires a new `emissary-core/**` or SAM API;
- exact configured rate semantics cannot be represented within a defensible hard memory ceiling and cannot be rejected before allocation;
- applying `leaseSetEncType` requires restoring control-plane `STREAM FORWARD` or a Yosemite/protocol fork;
- HTTP `100 Continue` must be fully supported rather than explicitly rejected;
- IRC inactivity cannot be implemented without parsing/reframing post-registration traffic;
- Streamr compatibility is proven to require unauthenticated non-loopback exposure;
- a new I2PControl-only dependency violates M062/M063 ownership;
- M079 finds any remaining high/medium issue.

## 13. Final closure rule

The tunnel security workstream remains `corrective pass required` until M080, M081, M082, M077, and M078 are independently closed and M079 accepts the actual final head.

Historical M073-M076 closure evidence remains useful for what those pinned commits demonstrated, but it does not override later regressions or independent findings. M079 is the final authority for closing this line of work.
