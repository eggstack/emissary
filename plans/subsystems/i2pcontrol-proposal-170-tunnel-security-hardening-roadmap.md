# I2PControl Proposal 170 Tunnel Security Hardening Roadmap

Status: corrective pass required; M084 ready; M085 blocked

Original planning baseline: `04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`.

Post-M076 corrective baseline: `1618de172e7a78a193fc1bb117af269f31174030`.

Merged-head corrective baseline: `e8feb9a3240a5a7b9dd5cc22a4ada47a0d9991ae`.

Source runtime roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Canonical/internal authority:

- `plans/000-long-term-specification.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- ADR-0001, ADR-0002, ADR-0003;
- M061 source-containment and M062/M063 dependency-containment authorities;
- M072 runtime-reclosure history;
- M073-M083 tunnel-security implementation/closure history.

Pinned external contract:

- I2P Proposal 170, `I2PControl Expansion`, open revision created/updated `2026-05-20`.

External I2P/I2P+/Yosemite sources remain read-only behavioral/security evidence. No upstream issue, PR, review, submission, merge request, contribution preparation, repository write, or maintainer contact is authorized.

## 1. Purpose

The substantive Proposal 170 tunnel-runtime/security implementation is now largely present:

- M080/M083 provide transactional, bounded accepted-server admission and exact canonical trusted Destination identity;
- M081 preserves generic-server `leaseSetEncType` truthfulness;
- M082 preserves HTTP `Expect` rejection and canonical POST accounting;
- M077 provides bounded IRC target connect plus activity-resetting post-registration idle expiry;
- M078 provides loopback-only Streamr local UDP boundaries and reference-aligned fanout bounds.

However, current `master` was formed by merging two separately verified lineages:

1. an older M077/M078/M079 security lineage; and
2. a later M083 admission/trusted-identity corrective lineage.

The historical M079 closure did not audit that merged composition. The merge also retained a stale IRC test helper call and lost exact M062 closure-path bookkeeping. Planning/status documents disagree about which milestone is current.

This roadmap therefore adds two final bounded milestones:

- **M084** — merged-head integration/planning corrective;
- **M085** — independent merged-head tunnel-security reclosure.

No new tunnel feature work is authorized by this final sequence.

## 2. Current repository findings

### 2.1 M083 runtime correction is present

Current admission separates peer history from active-only state, removes no-history inactive peers, derives historical capacity from the tightest safe enabled aggregate bound with fixed-window overlap, and keeps an inactive-only authoritative expiry index.

Current trusted peer identity parses with `Destination::parse_frame`, requires zero remainder, canonicalizes full Destination text from parsed bytes, and uses the 32-byte cryptographic Destination ID for accounting.

These mechanisms are retained.

### 2.2 M077 runtime correction is present

Current IRC server behavior includes:

- bounded registration;
- trusted peer-derived hostname rewrite;
- loopback local target;
- five-second target connection timeout;
- byte-transparent post-registration relay;
- ten-minute inactivity timeout reset by successful traffic in either direction.

The merge retained a test-only incompatibility: an admission-release test still calls the removed arbitrary-string `TrustedPeerIdentity::for_test` helper instead of an M083-valid structured fixture.

### 2.3 M078 runtime correction is present

Current Streamr behavior includes:

- loopback-only local client/server address policy;
- observed-loopback UDP source check;
- ten subscribers;
- 60-second expiry;
- 15-second refresh;
- one-byte control packets;
- 1200-byte application payload and 4095-byte transport receive bound;
- sequential bounded fanout.

These mechanisms are retained.

### 2.4 M079 is historical, not current-head authority

M079 independently reviewed and closed the older M077/M078 lineage. It did not review M083 or the later merge commit that combined M083 with that lineage.

Therefore M079 remains useful historical evidence but is insufficient as final current-head certification. M085 supersedes M079 only for final current-head reclosure authority.

### 2.5 M062/planning state is merge-inconsistent

Current M062 planning-path bookkeeping includes M083 closure evidence but not the merged M077/M078/M079 closure paths. Current planning/status documents also disagree about whether M083/M077/M078/M079 are ready, blocked, or closed.

M084 owns these integration/status defects.

## 3. Security/anonymity invariants

M084-M085 MUST preserve:

- exact Proposal 170 wire fields/actions/types/statuses;
- authenticated remote identity from SAM/Yosemite only;
- exactly one supported Destination with zero parser remainder;
- canonical downstream Destination text and 32-byte cryptographic accounting ID;
- transactional denial before attacker-owned peer-state mutation;
- finite global/per-peer concurrency and minute/hour/day peer/aggregate counters;
- peer-rate historical state only when enabled semantics require it;
- no-history inactive peer reclamation;
- capacity math across all enabled aggregate windows with fixed-window overlap and checked arithmetic;
- bounded peer map, expiry index, POST limiter, task groups, buffers, Streamr subscribers, and stop waits;
- generic accepted-stream raw relay and `leaseSetEncType` apply-or-reject;
- HTTP identity/proxy spoof stripping, response fingerprint stripping, unambiguous framing, fixed `Expect` rejection, and bounded POST accounting;
- IRC bounded registration/connect/idle occupancy with raw post-registration relay;
- Streamr loopback-only local boundary and bounded fanout;
- generation-local ephemeral state and stable backend-owned persistent server identity;
- no lock across network I/O/sleeps/joins;
- no private destination material in diagnostics;
- no timing-jitter/padding theater;
- no local DNS/LAN routing expansion;
- no new `emissary-core/**` production path;
- no startup/router/frontend ownership refactor;
- no new dependency or default feature widening;
- no hosted CI/fuzz/soak/release machinery for this bounded workstream;
- no upstream interaction.

## 4. Explicit non-goals

The final M084-M085 sequence does not authorize:

- new tunnel types or Proposal 170 API fields;
- new server admission algorithms;
- new HTTP features or informational-response state machines;
- new IRC protocol parsing after registration;
- generalized UDP/auth machinery;
- arbitrary I2CP/custom option pass-through;
- RouterInfo source-owner work;
- AddressBook/base-I2PControl expansion;
- core/router algorithm changes;
- public-network deanonymization experiments;
- upstream contribution preparation or review requests.

## 5. Dependency graph

```text
current merged head e8feb9a
        |
        v
M084 merged-head integration/planning corrective
        |
        v
M085 independent merged-head tunnel-security reclosure
        |
        v
security line closed only if M085 finds no high/medium issue
```

Historical sequence retained for traceability:

```text
M080 -> M081 -> M082 -> M083
                    \
                     +-- merged into current head
M077 -> M078 -> M079/
```

Dependency classification:

- merged-head review -> M084: corrective hard gate;
- M084 -> M085: hard gate;
- M085 final disposition: independent closure authority.

## 6. Milestone summary

### M080 — Admission transactionality/cardinality corrective

Closed with corrective history. Retain transactional denial, canonical peer keys, bounded state direction.

### M081 — Generic server LeaseSet option truthfulness

Closed. Retain accepted-stream `leaseSetEncType` application.

### M082 — HTTP peer identity / Expect / POST corrective

Closed with corrective history. Retain fixed-417 rejection and canonical POST key.

### M083 — Admission capacity and trusted Destination exactness

Closed for its implementation lineage and present in current `master`. Retain explicit peer history, tightest aggregate capacity proof, inactive-only expiry index, exact parse-frame consumption, and canonical text.

### M077 — IRC lifetime/exhaustion hardening

Implementation and closure are present. Runtime behavior is retained. Current merged-head test integration is not clean until M084 replaces the stale arbitrary-string trusted-peer fixture.

### M078 — Streamr local-boundary hardening

Implementation and closure are present. Runtime behavior is retained. Current merged-head containment bookkeeping is reconciled by M084.

### M079 — Historical integrated reclosure

Historical closure only. It certified the older M077/M078 lineage at its pinned head but did not audit M083 or current merge composition. It is not deleted or rewritten to claim otherwise.

### M084 — Merged-head integration and planning corrective

Status: **ready**.

Required scope:

- replace stale IRC test fixture with an existing M083 structurally valid peer fixture;
- restore exact M062 planning/closure path bookkeeping without broadening production scope;
- reconcile registry, roadmap, implementation README, and support/status docs;
- verify current merged integration compiles/tests cleanly;
- make no runtime behavior change;
- close with M085 as sole ready successor.

If M084 discovers a material production defect, create a separate narrow runtime corrective and keep M085 blocked.

### M085 — Merged-head tunnel-security reclosure

Status: **blocked on M084**.

Independently audit the actual post-M084 head. Rebuild current-head evidence for M083 admission/identity composed with M077 IRC, M078 Streamr, generic server, HTTP/httpbidir, lifecycle, option truthfulness, and containment.

Do not copy M079 assertions forward without current-head verification. Any high/medium finding creates a new corrective; M085 does not self-fix and self-certify material runtime defects.

## 7. M084 exit conditions

M084 may close only when:

- current I2PControl tests compile;
- stale `TrustedPeerIdentity::for_test` usage is removed from the IRC regression and replaced by a structurally valid M083 fixture;
- M061/M062 pass with exact-path bookkeeping and no widened production authority;
- M077/M078/M079 historical closure paths are represented by containment bookkeeping;
- planning/support status documents agree that final security closure is pending M085;
- full feature-enabled tests, feature-on/off checks, strict Clippy, scoped formatting, and diff checks pass;
- no runtime semantics changed;
- no new dependency/core/startup scope was introduced;
- no upstream interaction occurred.

## 8. M085 final evidence requirements

M085 must prove at the exact final head:

- both merge lineages plus M084 are present;
- trusted peer input is bounded, exact, canonical, and redacted;
- admission denial/state/capacity/expiry invariants hold under current code;
- no-history churn cannot fill peer state;
- historical policies with unbounded aggregate arrival reject before allocation;
- all enabled aggregate limits participate safely in capacity proof;
- generic server remains accepted-stream/raw and `leaseSetEncType` truthful;
- HTTP spoof/fingerprint/framing/Expect/POST behavior remains correct on the M083 identity boundary;
- IRC five-second connect and ten-minute activity-resetting idle expiry compose correctly with M083 admission leases;
- Streamr remains loopback-only and fanout/control/payload state is bounded;
- persistent identity/ephemeral runtime generation behavior remains correct;
- option-capability matrix has no accepted-but-ignored runtime field;
- M061/M062/M063 containment remains intact;
- no current high/medium security, anonymity, correctness, lifecycle, option-truthfulness, or containment finding remains.

## 9. Verification discipline

Use focused deterministic local tests and the full package suite:

- admission/trusted identity;
- generic server;
- HTTP/httpbidir;
- IRC server;
- Streamr;
- lifecycle/restart fixtures;
- M061/M062 containment;
- feature-disabled/enabled checks;
- core check;
- strict package Clippy;
- repository-accepted scoped nightly rustfmt;
- `git diff --check`.

Use fake/local SAM, TCP, and UDP fixtures only. No public-network test or hosted CI requirement is introduced.

## 10. Stop conditions

Stop and create a separate corrective plan if:

- a production runtime high/medium defect is found;
- trusted identity correctness would require weakening exact Destination validation;
- containment can pass only through broad production-glob expansion;
- a new core/Yosemite API or dependency is required;
- exact configured rate semantics cannot be represented within the existing hard bound and cannot reject before allocation;
- IRC/Streamr compatibility is claimed to require broader local exposure;
- any Proposal 170 wire/API expansion appears necessary;
- any upstream interaction would be required.

## 11. Final closure rule

The Proposal 170 **tunnel runtime/security line** remains `corrective pass required` until:

1. M084 independently closes the merged-head integration defects; and
2. M085 independently accepts the actual post-M084 head with no high/medium finding.

If M085 closes, the tunnel runtime/security line is complete against the pinned Proposal 170 revision and current internal fork head.

That does not close the separately documented partial Proposal 170 source/truthfulness state, RouterInfo 37/1/5 disposition, M051 blocker, or unrelated AddressBook/base-I2PControl limitations.

No upstream review, acceptance, merge, adoption, or submission is implied or authorized.
