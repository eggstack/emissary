# M085 — Merged-Head Proposal 170 Tunnel Security Reclosure

Status: blocked — hard dependency M084 merged-head integration corrective

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`.

Historical reclosure evidence:

- M079: `plans/closure/i2pcontrol-proposal-170/079-closure.md` — historical older-lineage closure only;
- M083: `plans/closure/i2pcontrol-proposal-170/083-closure.md` — accepted shared admission/trusted-Destination corrective.

Planning baseline: `e8feb9a3240a5a7b9dd5cc22a4ada47a0d9991ae` before M084.

Classification: invariant / independent final reclosure.

## 1. Objective

Independently re-audit and, only if the evidence supports it, reclose the complete Proposal 170 tunnel-security workstream against the actual post-M084 merged repository head.

M085 supersedes M079 only as the **current-head final reclosure authority**. M079 remains historical evidence for the older M077/M078 lineage, but it did not audit the later M083 merge and therefore cannot certify the current combined implementation.

M085 is primarily evidence and reconciliation work. It must not become a feature or opportunistic bug-fix milestone.

## 2. Readiness

M085 is not dependency-ready until M084 closes.

Before starting M085, confirm all of the following:

- M084 closure exists and records a post-M084 commit SHA;
- the current head includes M083 admission/trusted-identity corrections;
- the current head includes M077 IRC lifetime hardening;
- the current head includes M078 Streamr local-boundary hardening;
- the stale IRC test API mismatch is gone;
- M061/M062 containment passes at the post-M084 head;
- active planning/status documents identify M085 as the sole final reclosure handoff.

If any prerequisite is false, M085 remains blocked.

## 3. Why a new independent reclosure is required

The historical M079 closure states that it independently reviewed the final head after M073-M078. That was true for its branch lineage, but current `master` later merged a separately developed M083 lineage containing material changes to the shared accepted-server admission and trusted-peer identity boundary.

Therefore the current tree contains a composition that M079 never audited:

- M083 exact/canonical trusted Destination semantics;
- M083 peer-history/capacity/expiry-index state machine;
- M077 IRC idle/connect behavior consuming that shared accepted-server boundary;
- M078 Streamr local-boundary changes;
- M076/M082 HTTP filtering and POST accounting consuming the same trusted identity.

Branch-local verification is not sufficient evidence for the merged composition. M085 must re-run and re-reason about the actual combined head.

## 4. Canonical authority

M085 is governed by:

- `plans/000-long-term-specification.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- ADR-0001, ADR-0002, ADR-0003;
- M061/M062/M063 containment authorities;
- M073-M083 implementation and closure history;
- the pinned Proposal 170 `2026-05-20` revision.

External I2P/I2P+/Yosemite material may be used read-only for behavioral comparison. No upstream write, review request, issue/PR mutation, contribution preparation, merge request, or maintainer contact is authorized.

## 5. Scope

Review the actual integrated production implementation of all twelve tunnel types:

Client-side:

- `client`;
- `httpclient`;
- `ircclient`;
- `socks`;
- `socksirc`;
- `connectclient`;
- `streamrclient`.

Server-side:

- `server`;
- `httpserver`;
- `httpbidirserver`;
- `ircserver`;
- `streamrserver`.

Review their shared runtime ownership, trusted identity, admission, lifecycle, persistence, option truthfulness, and containment seams.

Do not reopen unrelated RouterInfo 37/1/5, M051 source-owner blockers, AddressBook limitations, or unrelated base-I2PControl gaps except to preserve truthful final documentation.

## 6. Explicit non-goals

M085 MUST NOT:

- add features or tunnel types;
- change Proposal 170 wire/API spelling;
- implement previously rejected/underspecified options;
- redesign admission, HTTP, IRC, Streamr, Yosemite, or TunnelManager;
- add timing jitter, padding, public-network deanonymization experiments, or generalized WAF behavior;
- broaden local/LAN routing;
- add dependencies, hosted CI, fuzz farms, soak farms, benchmark gates, release machinery, or upstream contribution machinery;
- silently fix a material runtime finding and still call the same pass an independent reclosure.

Tiny documentation/test-name corrections may be made if they do not change semantics. Any high/medium production finding requires a new corrective plan and keeps M085 open.

## 7. Required independent review areas

### 7.1 Merge ancestry and evidence integrity

Record the exact post-M084 head and prove that it contains the relevant M077, M078, M083, and merge-integration commits.

Reconcile historical closure claims with actual code. Do not copy the M079 matrix forward unchanged. Any assertion that refers to the old 524-byte trusted identity or pre-M083 admission state must be rebuilt against current code.

### 7.2 Trusted peer identity

Prove at current head:

- production ingress comes from authenticated Yosemite/SAM accepted-stream identity;
- input text is bounded before decode;
- decoded bytes contain exactly one structurally supported `Destination` with zero remainder;
- downstream B64 text is canonicalized from parsed serialized bytes;
- accounting keys use the canonical 32-byte Destination ID;
- malformed, whitespace/control-containing, oversized, truncated, and trailing-byte identities fail before handler/local-target work;
- full Destination text and canonical IDs remain redacted from high-cardinality diagnostics;
- HTTP trusted B64/B32 injection corresponds to the exact parsed authenticated Destination.

### 7.3 Admission resource/fairness state

Prove at current head:

- global and per-peer concurrency are finite;
- one peer cannot monopolize all accepted capacity;
- minute/hour/day peer and aggregate counters preserve configured semantics;
- historical peer state exists only for enabled peer-rate history;
- no-history inactive peers disappear on final lease drop;
- every historical policy with unbounded aggregate arrival is rejected unless exact bounded representability is proven;
- capacity uses the tightest safe bound across all enabled aggregate windows and includes fixed-window boundary overlap;
- arithmetic is checked and cannot wrap into an accepted smaller capacity;
- aggregate denial occurs before new peer-state insertion;
- denied attempts are mutation-free except bounded expired-state reclamation;
- active peers are intentionally unindexed and inactive historical peers have exactly one authoritative expiry registration;
- peer map and expiry index remain hard bounded;
- active/throttled state is not evicted to admit fresh attacker-controlled identities;
- admission leases release on success, error, EOF, idle expiry, panic isolation, cancellation, abort, stop, and generation replacement.

### 7.4 Generic server and option truthfulness

Prove:

- generic `server` uses accepted streams and never regresses to blind `STREAM FORWARD`;
- admitted payload remains byte-transparent;
- `leaseSetEncType` reaches Yosemite session configuration when accepted;
- every other runtime-relevant field is applied or rejected before allocation;
- no recognized option is persisted and ignored;
- startup-managed ownership remains separate.

### 7.5 HTTP/httpbidir boundary

Prove:

- arbitrary `X-I2P-*`, `X-Forwarded-*`, and named proxy identity headers are removed before loopback delivery;
- only trusted peer-derived I2P identity is rebuilt;
- `Expect` is rejected with bounded fixed behavior before local target connect;
- request framing rejects ambiguity;
- response `Date`, `Server`, provider/cache/trace and hop-by-hop fingerprint headers remain stripped;
- POST accounting uses the M083 canonical ID and remains bounded/churn-safe;
- POST denials happen before local target connect where required;
- `httpbidirserver` consumes the exact same inbound HTTP filter/admission path;
- persisted public hosting destination is identity metadata only and cannot become a local target on restart.

### 7.6 IRC boundary

Prove the M077 behavior on the M083 shared identity/admission implementation:

- registration validation and trusted `USER` rewrite happen before local connect;
- wrong-protocol/malformed registration fails closed;
- target host remains loopback-only;
- local connect is bounded to five seconds;
- post-registration relay remains byte-transparent;
- ten-minute inactivity resets on successful traffic in either direction;
- there is no fixed total lifetime for active sessions;
- idle expiry/EOF/error/cancellation release the shared admission lease;
- DCC/WEBIRC and unsupported paths remain unavailable/non-bypassable;
- structurally valid M083 peer fixtures are used by all integration tests.

### 7.7 Streamr boundary

Prove:

- local server bind and client output targets are loopback-only;
- non-loopback values reject before session/socket/task allocation;
- observed UDP source is checked as loopback defense in depth;
- remote payload cannot select local destination;
- subscriber ceiling is exactly 10 with no attacker-driven eviction;
- refresh at capacity updates in place;
- expiry remains 60 seconds and client refresh 15 seconds;
- control packets remain one byte and malformed controls create no state;
- application payload cap remains 1200 and transport receive bound 4095;
- fanout is sequential/bounded with no per-packet unbounded task queue;
- restart clears subscribers while persistent server identity remains stable.

### 7.8 Lifecycle and generation ownership

For every real backend prove:

- validation precedes allocation;
- running is published only after actual runtime readiness;
- stop is idempotent;
- restart is full stop then new generation;
- stale generation tasks cannot mutate replacement entries;
- child tasks are bounded/drained/aborted within declared stop timeouts;
- persistent secret/public identity ownership remains backend-owned and redacted;
- ephemeral admission/POST/subscriber/task state does not cross generations;
- one failed StartOnLoad definition does not fail unrelated definitions.

### 7.9 Containment and dependency review

Compare the final head with the accepted Proposal 170 containment baselines and the M084 pre-fix head.

Required conclusions:

- production tunnel-security changes remain inside `emissary-cli/src/i2pcontrol/**` except historical explicitly accepted exceptions;
- no new `emissary-core/**` production path was introduced by M080-M085;
- no unrelated startup/router/frontend refactor was introduced;
- no new dependency or default-feature widening occurred;
- Tokio `test-util` remains dev/test-only where intended;
- M061/M062/M063 pass at final head;
- planning-path bookkeeping is exact rather than broad-glob relaxation;
- no hosted CI/release/fuzz/soak/public-network infrastructure was added for this workstream.

A current containment deviation is at least medium severity unless proven purely documentary/test-local.

## 8. Required adversarial/integration evidence

Reuse existing tests where they already prove the current behavior, but run them at the actual final head. Add only missing integration tests.

At minimum demonstrate:

- minute-only peer history + all aggregate limits unlimited rejects before runtime state allocation;
- hour/day aggregate constraints can be tighter than minute and capacity uses the tightest safe bound;
- fixed-window boundary overlap cannot under-budget retained identities;
- no-history fresh peer churn leaves no inactive peer table buildup;
- aggregate-denied fresh identities do not grow peer/expiry state;
- active peer surviving a historical expiry remains bounded and reindexes correctly on final drop;
- valid null-cert and supported key-cert Destinations pass and trailing bytes fail;
- generic server `leaseSetEncType` is observed in session setup;
- generic raw bytes remain unchanged after admission;
- HTTP spoof headers and response fingerprints are stripped;
- HTTP `Expect` rejection creates no local target connection;
- HTTP POST churn cannot reset active throttling state;
- IRC idle relay releases admission and active traffic survives beyond twenty minutes of paused time;
- Streamr non-loopback config rejects before allocation and the 11th subscriber fails without evicting the first ten;
- restart during active accepted connections leaves no stale admission/task state;
- published server destination cannot become HTTP/httpbidir local target metadata after restart.

Use local fake SAM/TCP/UDP fixtures only.

## 9. Verification commands

Record exact outcomes for at least:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol runtime::admission
cargo test -p emissary-cli --no-default-features --features i2pcontrol peer_identity
cargo test -p emissary-cli --no-default-features --features i2pcontrol server
cargo test -p emissary-cli --no-default-features --features i2pcontrol http
cargo test -p emissary-cli --no-default-features --features i2pcontrol irc_server
cargo test -p emissary-cli --no-default-features --features i2pcontrol streamr
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

Run repository-accepted scoped nightly rustfmt for any file M085 itself touches. Do not rewrite unrelated formatting drift.

No GitHub Actions/hosted CI evidence is required for this bounded internal workstream.

## 10. Option-capability reconciliation

Rebuild or verify the current integrated option-capability matrix for all twelve tunnel types. Every runtime-relevant option must have exactly one disposition:

- applied and evidenced;
- invalid/irrelevant and rejected;
- recognized but unsupported and rejected before allocation.

Explicitly recheck:

- `MaxConcurrentConns`;
- `ClientPerMinute/Hour/Day`;
- `TotalInPerMinute/Hour/Day`;
- `PostLimit`/`PostLimitTime`;
- access-list fields;
- `PerClientPeriod`, `TotalPeriod`, `TotalBanTime`;
- `FilterFilePath`;
- `UniqueLocalAddressPerClient`;
- `MultiHoming`;
- `leaseSetEncType` and declared LeaseSet/session options;
- raw `CustomOptions`/`i2cp` handling;
- local host/interface fields for server/IRC/Streamr families.

Persist-and-ignore is a failure.

## 11. Documentation reconciliation

Only after the final evidence supports closure, reconcile:

- `docs/i2pcontrol/proposal-170-support.md`;
- `docs/i2pcontrol/tunnel-manager.md`;
- `docs/i2pcontrol/tunnel-backends.md`;
- `docs/i2pcontrol/streamr-runtime.md` if needed;
- tunnel-security roadmap;
- implementation README;
- `plans/registry.md`.

Documentation must state that tunnel-security reclosure is complete while Proposal 170 remains partial only for the separately accepted source/truthfulness limitations such as RouterInfo 37/1/5/M051.

Do not imply upstream review or acceptance.

## 12. Closure record requirements

Create `plans/closure/i2pcontrol-proposal-170/085-closure.md` only after the independent review is complete.

It MUST include:

- exact final commit SHA;
- exact relevant commit/merge ancestry including M077, M078, M083, M084;
- explanation of why historical M079 was insufficient for the merged head;
- fresh requirement-to-evidence matrix rather than copied M079 assertions;
- exact commands and outcomes;
- trusted identity review;
- admission capacity/state-machine review;
- HTTP/IRC/Streamr integration review;
- option-capability matrix;
- lifecycle/contention review;
- containment/dependency diff review;
- changed-path summary;
- unresolved findings with severity;
- final disposition;
- explicit read-only external research / no-upstream-interaction attestation.

## 13. Acceptance criteria

M085 may close only when:

1. M084 is independently closed;
2. the exact reviewed head includes both M083 and M077/M078 lineages plus M084 integration corrections;
3. full current-head I2PControl tests pass;
4. focused admission/trusted-peer/generic-server/HTTP/IRC/Streamr tests pass;
5. M061/M062/M063 containment is intact;
6. trusted peer identity is exact, canonical, bounded, and redacted;
7. admission state is transactional, hard-bounded, representable, and fair under configured semantics;
8. generic server option truthfulness remains intact;
9. HTTP identity/fingerprint/framing/POST behavior remains bounded and correct;
10. IRC idle/connect hardening composes correctly with M083 admission/identity;
11. Streamr local/fanout bounds remain correct;
12. restart/generation/persistence semantics remain correct;
13. no new production core/startup/frontend scope exists;
14. support documentation is truthful and internally consistent;
15. no high/medium security, anonymity, correctness, lifecycle, option-truthfulness, or containment finding remains;
16. no upstream interaction occurred.

## 14. Corrective disposition rule

Any high/medium finding means M085 does **not** close. Record `corrective pass required`, create a new narrowly scoped successor plan for the exact defect, and keep the tunnel-security roadmap open.

Do not patch a material finding inside M085 and then self-certify the result. The corrective must land and close before a fresh final reclosure is attempted.

Low-severity documentation/test naming drift may be corrected during M085 only when it has no runtime/security effect and is explicitly recorded.

## 15. Final workstream disposition

If M085 closes with no material finding, the Proposal 170 **tunnel runtime/security line** is complete against the pinned contract and current internal fork head.

That does not change the separately documented partial Proposal 170 source/truthfulness status, RouterInfo 37/1/5 disposition, M051 blocker, or any unrelated AddressBook/base-I2PControl limitation.

No upstream review, merge, adoption, or submission is implied or authorized.
