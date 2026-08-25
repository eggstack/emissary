# M089 Closure — Post-Corrective Tunnel Security Reclosure

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/089-post-corrective-tunnel-security-reclosure.md`

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Corrective predecessor closures:

- M087: `plans/closure/i2pcontrol-proposal-170/087-closure.md` — generic
  server progress-based inactivity correction;
- M088: `plans/closure/i2pcontrol-proposal-170/088-closure.md` — evidence-only
  Tier 3 disposition for unavailable lower-layer admission controls.

Planning baseline: `2b01bfd11ebcd768fcd5488f18b063ac336931a2`.

Reviewed implementation head: `115dc20` (`docs(i2pcontrol): close M088 and
ready M089`). M089 made no production runtime change; the reviewed production
head is therefore the corrected M087 head carried through the M088 planning
transition.

Review date: 2026-08-25.

## 1. Disposition

M089 independently re-audited the twelve registered Proposal 170 tunnel
backends from the perspective of an active remote I2P adversary. The evidence
supports closure without a new high- or medium-severity production finding
inside the approved Proposal 170 boundary.

M089 supersedes M085 only as the current-head tunnel runtime/security
reclosure authority. M085 remains valid historical evidence for its pinned
head, and M086 remains valid documentation/evidence reconciliation for its
scope.

The lower-layer pre-accept limitation remains explicit: the earliest
enforceable Emissary-owned application bound is after Yosemite/SAM
`Session<style::Stream>::accept()` returns. M089 does not claim that the
application admission state protects lower-layer stream-establishment work.

## 2. Server-family threat and evidence matrix

| Family/type | Threat path reviewed | Current bound/evidence | Result |
|---|---|---|---|
| Common accepted-server path: `server`, `httpserver`, inbound `httpbidirserver`, `ircserver` | Sybil peers open valid streams, send malformed SAM identity, or exhaust handler/peer state before local-target policy runs | `accepted_server.rs:115-135` accepts through Yosemite, then `TrustedPeerIdentity::from_stream`, then transactional `ServerAdmissionState::try_acquire`, then bounded `BoundedTaskGroup` handler ownership. Denials do not invoke handlers or allocate local targets. | pass; application admission is bounded and transactional, but intentionally post-accept |
| Trusted peer identity | Remote-controlled text attempts to multiply keys, retain trailing bytes, or spoof application identity | `peer_identity_impl.rs:53-105` bounds text, rejects controls/whitespace, parses exactly one `Destination` with zero remainder, canonicalizes full Base64 text, and derives a 32-byte cryptographic ID. `Debug` redacts both fields at `123-128`. | pass |
| Admission state | Denial churn, peer cardinality, stale generations, or lease leaks | `admission.rs:519-607` performs checks and reservation under one mutex before commit; `MAX_PEER_ENTRIES` and checked policy capacity bound state; final lease drop at `631-670` removes no-history peers or maintains one inactive expiry entry. Each runtime generation constructs fresh state. | pass |
| Lower-layer/pre-accept admission | Many valid Destinations consume streaming-manager, routing, and SAM/Yosemite work before application rejection | M088 source mapping found no Yosemite streaming-admission option and no consuming Emissary lower-layer limiter. Candidate `StreamConfig` fields are not an implemented consumer. | accepted residual limitation; Tier 3, out of M089 scope |
| Generic `server` | Valid peers pin finite admission slots with zero-progress raw relay | `server.rs:34-35,491-586` keeps loopback target confinement and a five-second target-connect timeout, then uses a ten-minute inactivity deadline reset only by successful byte transfer. EOF half-closes the opposite write direction; lease ownership remains in the handler task. | pass; M087 correction retained |
| HTTP server family | Request smuggling, spoofed proxy/I2P identity, body stalling, response fingerprinting, or POST-map churn | `filters/http.rs:179-299` bounds request line/headers and rejects ambiguous framing, upgrades, and all `Expect`; identity/proxy headers are stripped and trusted peer identity is injected. `copy_body`/`copy_response_body` use the 60-second deadline (`372-406`). `http_server.rs:117-158` bounds POST state at 1024 canonical peer keys and fails closed when full. Both HTTP server roles share `make_accepted_handler`. | pass; no new high/medium defect; no speculative byte cap added |
| `ircserver` | Registration line flood, spoofed presentation, target-connect pinning, or idle established session | `irc_server.rs:30-35,295-399,401-466` bounds registration to 12 lines, 1024-byte lines, five-second line reads, and 15 seconds total; rewrites the trusted hostname; connects only loopback with a five-second bound; raw relay expires after ten minutes of no successful progress. DCC/CTCP filtering remains bounded in `filters/irc.rs`. | pass |
| `streamrserver` | Subscriber-state exhaustion, remote datagram-selected local destination, payload amplification, or unbounded fanout | `streamr.rs:41-58,274-330,399-472` binds receive buffers to 4095 bytes, payloads to 1200 bytes, subscribers to 10, expiry to 60 seconds, refresh cadence to 15 seconds, local UDP to loopback, and fanout to a sequential snapshot loop. Remote datagrams select only authenticated subscriber destinations, never local UDP targets. | pass; reference-aligned specialized availability limitation recorded below |

The remaining client-side types were also checked for truthful registration and
ownership in `registry.rs:186-273`, plus their backend option validation and
local-boundary checks:

| Type | Evidence summary | Result |
|---|---|---|
| `client` | Yosemite outbound session, bounded local listener/task ownership, validated destination/port and fail-before-allocation options | pass |
| `httpclient` | bounded HTTP request parsing/body relay, I2P-only target policy or explicit configured outproxy, loopback listener validation | pass |
| `ircclient` | bounded local listener and shared IRC filter with fail-closed DCC/unsafe framing | pass |
| `socks` | bounded SOCKS parsing, loopback listener, I2P destination or explicit outproxy route, local-target rejection | pass |
| `socksirc` | composition of SOCKS and the accepted IRC filter without an unfiltered bypass | pass |
| `connectclient` | bounded CONNECT parsing and I2P/outproxy route selection with local-target rejection | pass |
| `streamrclient` | loopback-only configured UDP target, fixed destination/port tuple, 4095-byte receive buffer and 1200-byte forwarding cap | pass |

All twelve types remain registered exactly once. Unsupported options continue
to fail before session/listener allocation rather than being persisted and
ignored. No tunnel type, JSON-RPC field, or wire contract was added by M089.

## 3. Exact M088 lower-layer disposition

M088 is accepted as an evidence-only Tier 3 disposition. Yosemite 0.7.0
exposes `Session<style::Stream>::accept()` but no lower-layer concurrent or
rate admission configuration. Emissary's similarly named streaming config
fields are declarations/defaults without a consuming limiter, and accepted
streams are created with the default lower-layer configuration. Passing Java
option names through the existing SAM option map would therefore be
persist-and-ignore behavior, not pre-accept enforcement.

The residual work includes authenticated SYN parsing, signature/replay checks,
pending/active stream-manager state, routing-path binding, stream task/channel
state, and local SAM socket/session work before application admission. This is
an accepted availability and load/timing-correlation limitation. A future
lower-layer correction would require a separately authorized streaming
algorithm or dependency boundary; M089 does not open or imply that work.

## 4. HTTP residual-risk disposition

The HTTP server family retains bounded request-line/header parsing, duplicate
and conflicting `Content-Length` rejection, `Transfer-Encoding` rejection,
fail-closed upgrade/`Expect` handling, trusted peer-derived identity injection,
proxy/I2P identity stripping, response fingerprint stripping, a finite
approximately 60-second request/response body relay deadline, and a hard-bounded
fail-closed POST limiter keyed by the canonical 32-byte peer ID.

No request-body byte cap was added. The existing deadline, streaming relay, and
global application admission bound occupancy without introducing a new
compatibility policy. The bounded POST table can be pressured by Sybil
Destinations, but it refuses new entries at capacity and does not evict active
state; no new high/medium fork defect was demonstrated. No token bucket,
randomized delay, padding, or fairness replacement was introduced.

## 5. IRC and Streamr residual-risk disposition

IRC retains bounded registration, trusted peer-derived presentation, local
target allocation only after registration validation, a five-second target
connect bound, a ten-minute activity-resetting established-session inactivity
expiry, bounded unsafe DCC/CTCP handling, and raw post-registration relay.

Streamr retains the current pinned M078/M085/M089 model: ten subscribers, 60
seconds of expiry, approximately 15 seconds of refresh, 4095-byte transport
buffer, 1200-byte application payload, loopback-only local UDP, and sequential
bounded fanout. Multiple attacker-controlled Destinations can occupy and
refresh all ten slots. Java I2P and I2P+ use the same ten-subscriber/
60-second reference model, so this is a specialized reference-aligned
availability limitation rather than a fork regression. Random eviction or a
new public authentication/allowlist mechanism is not justified by this review.

There is a low-severity planning-document discrepancy: tracked `AGENTS.md` and
the older M072 capability matrix contain a 16-subscriber statement, while the
current M078 corrective plan, current support docs, implementation, tests, and
M089's pinned acceptance criteria use ten. M089 does not rewrite historical
M072 evidence or change production code. Planning owner disposition: reconcile
the stale 16-subscriber wording in a future documentation decision if 16 is
intended; no runtime security blocker is created for the current pinned
M078-M089 contract.

## 6. Cross-cutting anonymity and lifecycle review

- Server targets are loopback-confined where the Proposal 170 contract expects
  local service forwarding; Streamr remote datagrams cannot choose a local
  destination.
- HTTP proxy/I2P identity headers are stripped and trusted identity is derived
  only from the authenticated peer object. IRC presentation is derived from
  the same peer object. No clearnet address substitutes for a Destination ID.
- `StoredDestination` has redacted `Debug`/`Display`; backend status and runtime
  errors expose only public destination material or sanitized generic text.
  Persistent server identity uses the fixed backend-owned store path and
  rejects request-controlled path material.
- Admission, POST, subscriber, task, buffer, and runtime-supervisor state are
  bounded. Runtime generation state and admission state are ephemeral and do
  not cross restart generations.
- No reviewed path holds a mutex across network I/O, sleeps, or task joins.
  Handler/task ownership releases admission on normal completion, EOF, idle
  expiry, error, panic isolation, cancellation, and bounded stop.
- All twelve registered tunnel types remain truthful to their current runtime
  behavior; unsupported features remain explicit unsupported/unavailable
  responses.

## 7. M087/M088 changed-path containment

The corrective range was compared from the M089 planning baseline through
reviewed head `115dc20` and split by corrective ownership:

| Corrective | Path | Classification | Containment result |
|---|---|---|---|
| M087 | `emissary-cli/src/i2pcontrol/backends/server.rs` | production generic-server relay and focused tests | allowed `i2pcontrol` path; required for inactivity correction |
| M087 | `plans/closure/i2pcontrol-proposal-170/087-closure.md`; M087 plan/bookkeeping | planning evidence | allowed bookkeeping |
| M088 | no production runtime path | source/revision evidence only | no dependency/core/router change; Tier 3 stop condition honored |
| M088 | M088 closure/plan plus M062 test bookkeeping | planning/evidence | allowed bookkeeping |
| M089 | closure, registry, roadmap, and plan status updates | planning/closure only | no production runtime path changed |

No dependency declaration, root manifest, lockfile, `emissary-core/**`,
router, startup, or frontend path was changed by M087/M088. The M062 guard
remains the authoritative dependency-containment check.

## 8. Verification

Commands executed against the reviewed head:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

Results:

| Command | Result |
|---|---|
| Full `emissary-cli` i2pcontrol test suite | pass — 1,688 tests across 24 suites |
| M062 dependency containment | pass — 19 tests |
| `git diff --check` | pass |
| Targeted source inspection | pass — accepted-server order, M087 inactivity, HTTP, IRC, Streamr, identity redaction/persistence, registry truthfulness, and containment reviewed |

The required formatter check was also attempted with stable and nightly
toolchains. Both report pre-existing formatting differences in untouched
repository files (including `emissary-cli/src/address_book.rs` and existing
I2PControl modules); no unrelated reformatting was introduced for M089.

## 9. Findings and deferred hardening

| Severity | Finding | Owner/disposition |
|---|---|---|
| medium, accepted residual | No lower-layer/pre-accept admission limiter exists at the current Yosemite/SAM/core boundary. | M088 Tier 3 disposition; accepted out of M089 scope. A future implementation requires a separately authorized dependency-boundary or streaming plan. |
| low, planning-only | 16-subscriber wording in `AGENTS.md`/old M072 matrix conflicts with the current ten-subscriber M078-M089 contract and implementation. | Planning owner; future documentation reconciliation if 16 is intended. No production change in M089. |

The following remain intentionally deferred and are not M089 blockers:

- process-wide admission budgets shared across independently configured server
  tunnels;
- replacing fixed minute/hour/day windows with token-bucket or GCRA accounting;
- randomized rejection timing, jitter, or padding;
- global Sybil-resistant identity economics;
- generic zeroization unrelated to a demonstrated secret-lifetime defect;
- portability hardening outside the Proposal 170 runtime path;
- stronger public Streamr authentication, allowlists, or fairness semantics
  without compatibility evidence.

No high-severity finding remains. No new high/medium production finding was
confirmed inside the approved boundary.

## 10. Final registry and roadmap disposition

The following planning state is accepted in the M089 closure transition:

- M089 implementation plan: `closed`;
- M089 closure: this document;
- M085: retained as historical pinned-head evidence and superseded only as
  current-head authority;
- tunnel-security roadmap: `closed; M089 current-head reclosure accepted`;
- registry: no tunnel-security handoff remains dependency-ready or active;
- implementation handoff README: M087-M089 closed, with no future
  dependency-gated handoff currently registered;
- unrelated M051 RouterInfo blocker and separately partial Proposal 170,
  AddressBook, and base-I2PControl limitations remain unchanged.

The audit found no existing future tunnel-security plan to unblock. No new
future plan is required for the accepted M088 limitation or the documented
Streamr Sybil availability property. The low-severity 16-versus-10 planning
wording is recorded for a future decision rather than promoted to an active
runtime handoff.

## 11. Internal-only attestation

All external specifications, dependency source, and I2P/I2P+ reference
implementations used by the predecessor evidence were read-only evidence.
M089 performed no public-network deanonymization, load, or interoperability
experiment. No upstream repository, issue, pull request, review, merge
request, discussion, maintainer channel, submission, contribution artifact, or
external write was opened, drafted, mutated, or requested. All repository
writes remain internal to `eggstack/emissary`.
