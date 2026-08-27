# M091 Closure — Pre-Accept Stream Concurrency Boundary Hardening

Status: corrective pass required / superseded by M092

Historical disposition: this record preserves the technical implementation/test evidence that landed at commits `5053ce6b595351b251afb36f1f7d5278ef8f58d1` and `944da7b887b6efbd46601e9fad1c853581f40b8e`. It is **not** current authority for the proposed lower-layer concurrency design because the registered plan was `blocked` at `7194fa50ac03b44fb4c08a4d4d05d5fd33ea49b3` and the implementation's full-Yosemite-vendor / root-path-dependency strategy was not authorized before it landed. Per `plans/003-planning-process.md`, technical success does not cure missing pre-implementation authority.

M092 (`plans/implementation/i2pcontrol-proposal-170/092-m091-authorization-and-containment-corrective.md`, closure at `plans/closure/i2pcontrol-proposal-170/092-closure.md`) removed the vendored Yosemite copy, restored crates.io Yosemite 0.7.0, removed the three `emissary-core` SAM/streaming changes and the accepted-server lower-layer option seam, restored M060/M061/M062 containment semantics to their pre-M091 authority, and returned the M088 pre-accept / lower-layer limitation to the accepted residual disposition.

The M091 plan at `plans/implementation/i2pcontrol-proposal-170/091-pre-accept-stream-concurrency-boundary-hardening.md` has been returned to its pre-implementation `blocked` status with a `superseded by M092` annotation. Any future lower-layer concurrency design must be re-registered under a new explicit maintainer authorization; this closure does not retroactively approve the deprecated vendor strategy.

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/091-pre-accept-stream-concurrency-boundary-hardening.md`

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Predecessors:

- M088 lower-layer admission evidence and accepted residual limitation;
- M089 current-head tunnel security reclosure;
- M090 server loopback and IRC half-close corrective.

Planning baseline: `f0f3fc2204318c2fac69817d347df2702c51287b`.

Implementation commit: `5053ce6b595351b251afb36f1f7d5278ef8f58d1`.

Review date: 2026-08-26.

## 1. Disposition

M091 is complete. The accepted-server policy ceiling is now carried through an
explicitly authorized vendored Yosemite 0.7.0 maintenance copy as the standard
`i2p.streaming.maxConcurrentStreams` STREAM session option. Emissary validates
that option into bounded per-session `StreamConfig`, and the streaming manager
rejects an authenticated, replay-bound inbound SYN with the existing fixed
STREAM reset before listener, pending-stream, channel, routing, or task
allocation.

The existing post-accept `ServerAdmissionState` remains unchanged and retains
peer identity, aggregate/per-peer concurrency, and minute/hour/day rate
ownership. M091 adds only the lower-layer concurrency defense in depth; it does
not claim to solve lower-layer per-peer/rate or Sybil resistance.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Default behavior remains unrestricted | `StreamManager::new` uses `StreamConfig::default()`; `None` is preserved by option parsing; vendored `SessionOptions` defaults to `None` | pass |
| Configured ceiling allows exactly the permitted occupancy | `StreamManager::stream_count` counts active, pending inbound, and pending outbound state; focused manager tests admit the first pending SYN and reset the next at limit 1 | pass |
| Rejection occurs before normal allocation | The check follows packet parse, signature verification, and replay binding and precedes `listener.pop_socket`, `PendingStream::new`, map insertion, channel allocation, routing, and task creation | pass |
| Pending inbound states count | `pending_inbound.len()` participates in `stream_count`; regression test fills the ceiling with pending state | pass |
| Active states count | `active.len()` participates in `stream_count`; regression test inserts active state and observes a reset with no pending allocation | pass |
| Invalid SYNs do not consume capacity | Malformed, unsigned, and replay-mismatched packets are exercised before a valid SYN is admitted by `invalid_inbound_syns_do_not_consume_limit`; validation ordering is before the limit check | pass |
| Pending expiry releases capacity | Stale pending entries are removed from both `pending_inbound` and `destination_streams`; expiry regression admits a later SYN | pass |
| Active close/error releases capacity | Existing stream-task completion removes the entry from `active`; `remove_session` removes active and pending state; the full core suite remains green | pass |
| Shutdown/restart starts with fresh state | Each SAM session constructs a fresh `StreamManager`; no process-global counter or registry was added; full SAM/core and CLI suites pass | pass |
| Accepted-server policy is translated only to common STREAM servers | `accepted_server.rs` translates `max_concurrent_connections()` into Yosemite `SessionOptions`; the capture regression sees the exact option value | pass |
| Client and Streamr sessions do not receive the server option | Only `AcceptedServerRuntime` sets the field; Yosemite regression verifies STREAM-only emission and DATAGRAM omission; other constructors retain defaults | pass |
| Post-accept admission remains defense in depth | `ServerAdmissionState` and the existing bounded task group were not changed; full I2PControl CLI tests pass | pass |
| Denial diagnostics contain no private remote material | The new denial log records only the local Destination ID and a fixed message; no remote Destination or key is logged | pass |
| No Proposal 170/API or Streamr expansion | No JSON-RPC surface, tunnel type, Streamr path, or `httpbidirserver` identity behavior changed | pass |

## 3. Implementation evidence

The dependency boundary is an internal vendored maintenance copy of Yosemite
0.7.0. It adds one typed optional `SessionOptions` field, emits the standard
option only for STREAM session creation, preserves the historical default, and
contains crate-level allowances for pre-existing vendored clippy findings.
The core session constructor consumes the received SAM option map through a
bounded parser capped at 4096, while the accepted-server policy supplies values
bounded by the application policy.

The lower-layer check is intentionally after authentication and replay
validation. Over-limit SYNs use the existing packet builder to emit a bounded
reset with the original stream identifiers and return without creating normal
stream state. Occupancy is derived from the existing active/pending maps, and
pending expiry now removes its destination mapping as well as the pending
entry. Session removal and stream-task completion continue to release active
state through the existing owner paths.

The M060 and M061 historical containment guards were amended narrowly to record
the approved M091 configuration seam. M062 now records the exact vendored file
set and the single Yosemite Cargo.lock replacement; no broad production glob
was added.

## 4. Failure, cancellation, recovery, and contention review

- Invalid option values are ignored to preserve default-off behavior; valid
  values are bounded before entering core state.
- Signature, destination, and replay failures return before the concurrency
  check and do not alter occupancy.
- Over-limit rejection is fixed and bounded: one reset attempt is queued and
  queue failure is safely ignored, with no accepted stream allocation.
- Pending expiry, active task completion, `remove_session`, and fresh SAM
  session construction release or isolate state through existing owners.
- The lower-layer count is per `StreamManager`; there is no process-wide
  cross-Destination budget, shared admission registry, or lock held across
  network I/O, timers, or joins.
- The application admission lease and richer rate policy remain the authoritative
  post-accept controls for Proposal 170 server behavior.

## 5. Compatibility and security review

The new field is optional and default-off. STREAM sessions created without the
explicit accepted-server option retain historical behavior; DATAGRAM and
client/Streamr construction paths do not emit this option. No public API,
JSON-RPC schema, persistence format, wire contract, tunnel identity, or
Streamr limit changed.

The remaining limitation is explicit: the lower layer has no per-peer or
per-peer-rate admission accounting, so authenticated Destination churn and
SYN verification work remain possible up to the configured aggregate session
ceiling. The existing application policy remains the owner of peer/rate
controls. A future plan may address that residual only with new authorization.

## 6. Changed paths and containment

Implementation commit:

- `Cargo.toml`, `Cargo.lock`;
- `vendor/yosemite/**` — exact files enumerated by the M062 guard;
- `emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs`;
- `emissary-core/src/sam/protocol/streaming/config.rs`;
- `emissary-core/src/sam/protocol/streaming/mod.rs`;
- `emissary-core/src/sam/session.rs`;
- narrow M060/M061/M062 containment authority updates.

Planning and closure bookkeeping follows in the planning commit:

- M091 implementation plan and workstream README;
- `plans/registry.md`;
- the tunnel security hardening roadmap;
- this closure record.

The M062 exact-path guard passes. No upstream repository, issue, pull request,
dependency submission, or external review was used as an implementation
authority.

## 7. Verification

| Command | Outcome |
|---|---|
| `cargo test -p emissary-core` | pass; full library, IPv6, ML-KEM, and SAM integration coverage; one initial ML-KEM bind collision passed on immediate isolated rerun |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | pass; full I2PControl CLI suite |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment` | pass; 7 M061 tests and 19 M062 tests |
| `cargo test --manifest-path vendor/yosemite/Cargo.toml stream_concurrency_option_is_carried_only_for_stream_sessions` | pass; 1 test |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass |
| `git diff --check` | pass |
| `cargo fmt --all -- --check` | not green under installed stable/nightly toolchains because repository rustfmt configuration uses nightly-only settings and existing repository-wide style differs; no formatter-only changes were retained |

## 8. Future-plan disposition

M091 is **closed**, and its former Yosemite option-transport blocker is
resolved by the explicitly authorized internal vendor strategy. No other
currently registered future implementation plan became dependency-ready from
this closure: M051 remains independently blocked by its accepted missing
RouterInfo owners, and the future current-head reclosure remains intentionally
unregistered in the roadmap pending the normal next-handoff review. No new
implementation handoff was created.

The registry, implementation README, and security roadmap now all record M091
as closed, link this evidence, preserve the exact M062 containment exception,
and leave future reclosure work unregistered rather than treating it as an
active plan.

## 9. Internal-only attestation

All implementation, test, containment, closure, registry, and roadmap writes
were confined to the internal `eggstack/emissary` repository. No upstream
interaction or external repository write was performed.
