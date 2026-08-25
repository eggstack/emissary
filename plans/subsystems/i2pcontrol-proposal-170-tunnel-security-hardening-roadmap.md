# I2PControl Proposal 170 Tunnel Security Hardening Roadmap

Status: runtime/security closed by M085; M086 documentation/evidence reconciliation ready

Original planning baseline: `04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`.

Post-M076 corrective baseline: `1618de172e7a78a193fc1bb117af269f31174030`.

Merged-head corrective baseline: `e8feb9a3240a5a7b9dd5cc22a4ada47a0d9991ae`.

M084 post-fix baseline: `1196a4d85cecb4f9676a8d87d27c69322816d7a8`.

M085 final reviewed head: `a6f18268b8d8724ed826f69614161b5b8d293ef5`.

M086 planning baseline: `185d43174c491a57c217c39e45555d136f40a406`.

Source runtime roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Canonical/internal authority:

- `plans/000-long-term-specification.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- ADR-0001, ADR-0002, ADR-0003;
- M061 source-containment and M062/M063 dependency-containment authorities;
- M072 runtime-reclosure history;
- M073-M085 tunnel-security implementation/closure history.

Pinned external contract:

- I2P Proposal 170, `I2PControl Expansion`, open revision created/updated `2026-05-20`.

External I2P/I2P+/Yosemite sources remain read-only behavioral/security evidence. No upstream issue, PR, review, submission, merge request, contribution preparation, repository write, or maintainer contact is authorized.

## 1. Purpose and current disposition

The Proposal 170 tunnel runtime/security implementation is closed against the pinned contract and current internal fork head.

The substantive accepted runtime/security boundaries are:

- M080/M083 — transactional bounded accepted-server admission and exact canonical trusted Destination identity;
- M081 — generic-server `leaseSetEncType` apply-or-reject truthfulness;
- M082 — HTTP `Expect` rejection and canonical POST accounting;
- M077 — bounded IRC target connect plus activity-resetting post-registration idle expiry;
- M078 — loopback-only Streamr local UDP boundaries and bounded reference-aligned fanout;
- M084 — merged-head integration corrective;
- M085 — independent current-head final reclosure.

M085 found no high/medium security, anonymity, correctness, lifecycle, option-truthfulness, or containment defect at the reviewed post-M084 head.

M086 is a final documentation/evidence-integrity corrective only. It does not reopen runtime/security closure and authorizes no production-source change.

## 2. Resolved merged-head history

Current `master` descended from the merge of two separately verified lineages:

1. the older M077/M078/M079 security lineage; and
2. the later M080-M083 admission/identity corrective lineage.

M079 certified only the older lineage at its pinned head. M084 repaired the merged composition, including:

- stale IRC trusted-peer test fixture usage;
- a stale pre-M083 admission regression test;
- dropped `is_proxy_identity_header` / `is_i2p_identity_header` helper definitions whose call sites survived the merge;
- missing M062 exact-path planning/closure bookkeeping.

M085 then independently audited the actual post-M084 head. Those merge defects are resolved history, not current runtime findings.

A later documentation audit found only record-quality inconsistencies:

- stale pending/reopened wording in active planning surfaces;
- stale `Destination::parse` wording in user-facing trusted-peer documentation;
- one M085 `MAX_PEER_ENTRIES` arithmetic transcription error;
- a need to clarify that M084's HTTP helper restoration was a bounded production-source merge restoration, not a new runtime feature.

M086 owns only those documentation/evidence issues.

## 3. Security/anonymity invariants

The closed runtime/security workstream establishes and M086 MUST preserve:

- exact Proposal 170 wire fields/actions/types/statuses;
- authenticated remote identity from SAM/Yosemite only;
- bounded Base64 peer text, exactly one supported `Destination`, zero parser remainder, canonical full-Destination text, and 32-byte cryptographic accounting ID;
- transactional admission denial before attacker-owned peer-state mutation;
- finite global/per-peer concurrency and minute/hour/day peer/aggregate counters;
- historical peer state only when enabled peer-rate semantics require it;
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
- no new dependency or default-feature widening;
- no hosted CI/fuzz/soak/release machinery for this bounded workstream;
- no upstream interaction.

## 4. Explicit non-goals

Neither the closed runtime/security work nor M086 authorizes:

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

M086 additionally MUST NOT touch production source or Cargo/dependency state.

## 5. Dependency graph

Historical runtime/security sequence:

```text
M080 -> M081 -> M082 -> M083
                    \
                     +-- merged into current head
M077 -> M078 -> M079/
             |
             v
            M084 merged-head integration corrective
             |
             v
            M085 independent current-head reclosure
```

Current remaining record-quality sequence:

```text
M085 runtime/security closure accepted
             |
             v
M086 documentation/evidence reconciliation
             |
             v
no active tunnel-security handoff if M086 closes
```

Dependency classification:

- M085 -> M086: evidence/history dependency only;
- M086 does not gate runtime/security behavior;
- any production defect discovered during M086 creates a separate new runtime corrective and stops M086.

## 6. Milestone summary

### M074 — Shared admission hardening

Closed with corrective history; later admission corrections are owned by M080/M083.

### M075 — Generic accepted-stream migration

Closed; M081 repaired `leaseSetEncType` truthfulness.

### M076 — HTTP anonymity/POST hardening

Closed with corrective history; M082/M083 own later identity/Expect corrections and M084 restored helper bodies lost in merge.

### M077 — IRC lifetime/exhaustion hardening

Implementation and closure present. M084 reconciled its stale merged-head trusted-peer test fixture. Runtime behavior is accepted by M085.

### M078 — Streamr local-boundary hardening

Implementation and closure present. M084 reconciled merged containment bookkeeping. Runtime behavior is accepted by M085.

### M079 — Historical integrated reclosure

Historical older-lineage closure only. M085 supersedes it for current-head certification.

### M080 — Admission transactionality/cardinality corrective

Closed with corrective history; retained by M083/M085.

### M081 — Generic server LeaseSet option truthfulness

Closed and retained.

### M082 — HTTP peer identity / Expect / POST corrective

Closed with corrective history; retained by M083/M085.

### M083 — Admission capacity and trusted Destination exactness

Closed and retained. Current trusted identity uses `Destination::parse_frame`, requires empty remainder, derives `parsed.id()`, and canonicalizes full-Destination text from `parsed.serialize()`.

### M084 — Merged-head integration and planning corrective

Closed. It repaired merge-composition failures and restored dropped HTTP identity-header helper definitions. M086 will add a historical clarification distinguishing that production-source restoration from a new intended runtime semantic change.

### M085 — Merged-head tunnel-security reclosure

Closed and controlling for current-head runtime/security certification. It independently audited the post-M084 head and found no high/medium remaining issue.

### M086 — Post-M085 documentation and evidence reconciliation

Status: **ready**.

Scope:

- reconcile stale registry/roadmap/README current-state language;
- correct trusted-peer support documentation to the M083/M085 exact parser/canonicalization boundary;
- add a transparent M085 capacity-arithmetic erratum (`83,886`, not `81,920`);
- clarify M084's bounded production HTTP-helper merge restoration without reopening runtime closure;
- add exact M086 planning/closure paths to the M062 planning allowlist;
- make no production-source, Cargo, dependency, feature, or lockfile change.

## 7. M086 exit conditions

M086 may close only when:

- active planning surfaces agree that M085 runtime/security closure is accepted and M086 is documentation/evidence-only;
- no M077/M078 current-state text says merged-head integration is still pending;
- user-facing trusted-peer documentation matches `Destination::parse_frame` + zero remainder + `parsed.id()` + canonical Base64 re-encoding;
- M085 closure contains an explicit traceable correction from `81,920` to `83,886`;
- M084 closure explicitly records the HTTP-helper production-source restoration as a bounded merge-restoration deviation already independently accepted by M085;
- M062 exact-path bookkeeping includes M086 plan/closure paths without broadening production authority;
- changed-path review proves no production source/manifests/lockfile changed;
- M062 test and `git diff --check` pass;
- no upstream interaction occurred.

## 8. Verification discipline

For M086 use only proportional evidence:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
git diff --name-only <M086-baseline>..HEAD
```

Targeted text inspection must confirm stale current-state phrases and the incorrect capacity value are gone or explicitly superseded by errata.

Do not rerun the full runtime/security suite merely to correct records. If any production-source change becomes necessary, stop and create a new runtime corrective.

## 9. Stop conditions

Stop M086 and create separate corrective planning if:

- current production code contradicts M085's runtime/security conclusions;
- trusted-peer documentation cannot be corrected without changing implementation;
- the `83,886` correction reveals an actual runtime capacity error rather than a closure transcription error;
- an HTTP helper defect remains in current production code;
- containment can pass only through broad production-glob expansion;
- any Cargo/core/router/startup/runtime change is required.

## 10. Final closure rule

The Proposal 170 **tunnel runtime/security line remains closed by M085** throughout M086.

M086 closes only the residual documentation/evidence-integrity corrective. If M086 closes cleanly:

- no active tunnel-security handoff remains;
- M085 remains the current-head final runtime/security authority;
- Proposal 170 remains partial only for the separately documented source/truthfulness limitations, RouterInfo 37/1/5 disposition, M051 blocker, and unrelated AddressBook/base-I2PControl limitations.

No upstream review, acceptance, merge, adoption, or submission is implied or authorized.
