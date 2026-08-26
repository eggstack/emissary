# Emissary Active Planning Registry

This file is the compact control surface for active planning.

Canonical direction:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`

## Status vocabulary

- **proposed** — document exists but is not approved for execution.
- **ready** — dependencies and interfaces are satisfied; plan may be handed off.
- **active** — implementation or closure work is in progress.
- **blocked** — a named dependency or evidence requirement prevents progress.
- **closing** — implementation landed and independent closure evidence is being gathered.
- **closed** — closure record accepted for the pinned implementation head.
- **closed internally against pinned revision** — internal closure accepted against an explicitly named revision of an open external specification; does not imply upstream review or acceptance.
- **partial Proposal 170 support** — exact supported dimensions are closed, but one or more pinned source/runtime capabilities remain truthfully unavailable.
- **corrective pass required** — a prior disposition or closure was invalidated by a material implementation, compatibility, scope, merge-integration, security, or evidence defect.
- **superseded** — replaced by another document and not executable.
- **archived** — inactive and retained for traceability.

## Active subsystem roadmaps

| Subsystem | Status | Roadmap | Current handoff | Dependencies or blockers |
|---|---|---|---|---|
| I2PControl Proposal 170 source/truthfulness | partial Proposal 170 support; M057 closed | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | no source-completion handoff | M051 remains blocked by absent substantive news/ban owners; accepted RouterInfo matrix remains 37/1/5 |
| I2PControl Proposal 170 containment | accepted/closed authority | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | exact-path bookkeeping for M090/M091 | M061/M062/M063 semantics remain authoritative; no broad production containment expansion is authorized |
| I2PControl Proposal 170 tunnel runtime completion | functionally complete; security corrective sequence tracked separately | `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md` | no runtime-feature handoff | all twelve registered tunnel backends remain real; M090/M091 are security hardening, not new tunnel capability |
| I2PControl Proposal 170 tunnel security hardening | **reopened; M090 ready** | `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | **M090** | M091 is blocked by the absent Yosemite/SAM transport for pre-accept streaming concurrency policy |

## Canonical scope amendment for tunnel runtimes

ADR-0003 remains the controlling scope amendment for the Proposal 170 tunnel data planes. ADR-0001/ADR-0002 remain controlling for contract spelling, startup/control-plane separation, server secret ownership, and internal-only scope except where ADR-0003 explicitly superseded earlier data-plane deferment.

The preferred production boundary remains `emissary-cli/src/i2pcontrol/**`. M090 is required to remain inside that boundary. M091 is a separately scoped lower-layer corrective and is intentionally **blocked** because it would require an explicitly authorized narrow configuration path into the streaming implementation; its existence does not authorize arbitrary `emissary-core/**`, router, dependency, or lockfile changes.

No upstream review, merge, submission, contribution preparation, issue/PR mutation, or maintainer contact is authorized. External I2P/I2P+/Yosemite repositories and specifications are read-only evidence only. Repository writes remain internal to `eggstack/emissary`.

## Post-M089 corrective trigger

M089 remains valid current-head tunnel-security closure evidence for its pinned head `f0f3fc2204318c2fac69817d347df2702c51287b`. A later source review identified two narrow implementation defects plus one already-known medium lower-layer residual that now warrants explicit future planning.

The two directly executable defects are:

1. `httpserver` and `ircserver` accept the compatibility spelling `localhost` as a loopback target but retain it as a hostname until `TcpStream::connect`, so the claimed loopback-only invariant still depends on local resolver/NSS configuration;
2. `ircserver` terminates its post-registration relay when either direction reaches EOF, unlike the M087-corrected generic `server` relay, so useful TCP half-close/drain behavior is lost and termination timing differs unnecessarily.

These are combined into M090 because they are both small server-boundary corrections inside `emissary-cli/src/i2pcontrol/**` and share the same security/lifecycle verification surface.

The separate lower-layer residual is the M088 finding: common accepted-server admission runs only after Yosemite/SAM `Session<style::Stream>::accept()` returns. Signed-SYN parsing, replay verification, pending/active stream state, routing-path work, and local SAM/Yosemite work may occur before application rejection. Java I2P performs a streaming-layer concurrency/rate decision earlier in its inbound SYN path.

M091 now records the intended minimal hardening architecture, but it is not ready. Current Yosemite 0.7.0 and its current `master` expose no streaming-concurrency `SessionOptions` field, and Emissary's declared `StreamConfig::max_concurrent_streams` field is not currently a consumed per-session configuration path. M091 therefore cannot be implemented truthfully without first resolving that exact transport boundary.

The same review does **not** authorize changes to Streamr subscriber semantics or `httpbidirserver` identity sharing:

- Streamr retains the reference-aligned ten-subscriber / 60-second expiry model and its documented Sybil monopolization limitation;
- `httpbidirserver` retains the fork-local separate unpublished client session rather than adopting Java I2P's server-Destination-sharing behavior.

## Dependency-ready implementation plan

**M090 is the only dependency-ready tunnel-security implementation handoff.**

Plan:

- `plans/implementation/i2pcontrol-proposal-170/090-server-loopback-and-irc-half-close-corrective.md`

M090 may begin from planning baseline `f0f3fc2204318c2fac69817d347df2702c51287b`. It must:

- normalize accepted HTTP/IRC local targets to literal loopback `IpAddr` values before runtime connection;
- preserve the existing accepted option spellings without adding DNS/LAN target capability;
- correct IRC half-close/drain behavior while retaining the ten-minute progress-based inactivity bound;
- remain inside `emissary-cli/src/i2pcontrol/**` plus focused tests/planning bookkeeping;
- add no dependency, core/router/startup/frontend, Proposal 170 API, Streamr, or bidirectional-identity change.

## Blocked future implementation plan

M091 exists so the accepted M088 lower-layer limitation has an explicit owner and bounded target, but it is **blocked and not executable**.

Plan:

- `plans/implementation/i2pcontrol-proposal-170/091-pre-accept-stream-concurrency-boundary-hardening.md`

Exact blockers:

1. M090 must close first under the normal one-ready-handoff sequencing rule;
2. a supported configuration transport must exist from the accepted-server `ServerAdmissionPolicy` through Yosemite/SAM into the Emissary streaming manager before `accept()`;
3. if that requires a Yosemite dependency change, fork/vendor/git dependency, manifest/lockfile delta, or `emissary-core/**` expansion, the exact strategy and paths must be explicitly authorized before M091 moves to `ready`.

M091's target is deliberately narrower than Java parity: pre-accept stream **concurrency** defense in depth only. It does not pre-authorize duplication of per-peer minute/hour/day rate accounting into core.

## Current tunnel-security sequence

```text
M087 generic server inactivity corrective            [CLOSED]
  |
  v
M088 pre-accept boundary evidence                     [CLOSED / TIER 3]
  |
  v
M089 independent current-head security reclosure      [CLOSED @ f0f3fc2]
  |
  v
M090 resolver-free loopback + IRC half-close          [READY]
  |
  v
M091 pre-accept stream concurrency boundary           [BLOCKED]
  |
  v
future independent current-head security reclosure    [UNREGISTERED]
```

Per `plans/003-planning-process.md`, the future reclosure remains in the roadmap only. It is not registered while M091 is blocked.

## Recently closed tunnel-security authority

| Handoff | Current disposition | Plan | Note |
|---|---|---|---|
| M083 | closed | `plans/implementation/i2pcontrol-proposal-170/083-admission-capacity-and-trusted-destination-exactness-corrective.md` | current application-admission/trusted-Destination authority |
| M084 | closed | `plans/implementation/i2pcontrol-proposal-170/084-merged-head-integration-and-planning-corrective.md` | merged-head integration corrective |
| M085 | closed; historical pinned-head reclosure | `plans/implementation/i2pcontrol-proposal-170/085-merged-head-tunnel-security-reclosure.md` | historical evidence retained |
| M086 | closed; documentation/evidence only | `plans/implementation/i2pcontrol-proposal-170/086-post-m085-documentation-and-evidence-reconciliation-corrective.md` | no runtime change |
| M087 | closed | `plans/implementation/i2pcontrol-proposal-170/087-generic-server-inactivity-timeout-corrective.md` | progress-based generic relay inactivity + half-close behavior |
| M088 | closed; Tier 3 unsupported lower-layer semantic | `plans/implementation/i2pcontrol-proposal-170/088-pre-accept-server-admission-boundary-corrective.md` | lower-layer limitation remains technically valid evidence |
| M089 | **closed; current accepted reclosure for its pinned head** | `plans/implementation/i2pcontrol-proposal-170/089-post-corrective-tunnel-security-reclosure.md` | not rewritten; superseded as current-head authority only after a future reclosure closes |
| M090 | **ready** | `plans/implementation/i2pcontrol-proposal-170/090-server-loopback-and-irc-half-close-corrective.md` | sole dependency-ready handoff |
| M091 | **blocked** | `plans/implementation/i2pcontrol-proposal-170/091-pre-accept-stream-concurrency-boundary-hardening.md` | lower-layer transport/dependency boundary unresolved |

Older M064-M082 history remains in the implementation directory, closure records, and subsystem roadmaps and is not duplicated here.

## M090 scope guard

M090 may change only the target representation and IRC relay termination semantics required by the two post-M089 findings. Expected production paths are `http_server.rs`, `irc_server.rs`, and mechanically `http_bidir.rs` only if its shared HTTP handler/config type requires adjustment.

M090 must not:

- touch `emissary-core/**`;
- change dependencies or `Cargo.lock`;
- change accepted-server admission ordering or policy;
- broaden local targets beyond the existing loopback contract;
- change HTTP parser/filter/POST semantics;
- change IRC registration semantics;
- change Streamr;
- change `httpbidirserver` identity/session sharing;
- add Proposal 170 fields/types/actions;
- perform upstream interaction.

## M091 scope guard

M091 is planning authority only while blocked. It does not itself authorize production or dependency changes.

If later unblocked, its preferred architecture is:

- keep `ServerAdmissionPolicy` and all rich Proposal 170 admission business logic in `i2pcontrol`;
- carry only the already-validated concurrent-stream ceiling into the dedicated accepted-server session;
- enforce it after signed-SYN and replay validation but before `pending_inbound`, active stream/channel/task, routing-path, SAM accepted-socket, or Yosemite accepted-stream allocation;
- preserve default behavior for unrelated sessions when no limit is configured;
- retain post-accept `ServerAdmissionState` as defense in depth.

Vendoring/forking/patching Yosemite, switching to a git dependency, changing manifests/lockfile, or adding exact `emissary-core/**` paths requires a new explicit maintainer authorization/amendment before M091 may become ready.

## Durable tunnel-security invariants retained

The prior M074-M089 work remains authoritative for:

- exact Proposal 170 contract spelling/types/actions;
- authenticated SAM/Yosemite accepted-peer identity;
- exactly one parsed supported Destination with zero remainder and canonical downstream Base64 text;
- 32-byte cryptographic Destination ID for admission/POST accounting;
- transactional, bounded application admission with explicit peer-history semantics;
- no-history inactive peer reclamation and coherent inactive-only expiry index;
- HTTP spoof/fingerprint/framing/Expect/POST protections;
- IRC bounded registration, five-second target connect, ten-minute activity-resetting inactivity expiry, and raw post-registration relay;
- Streamr loopback-only local boundary, ten subscribers, bounded expiry/refresh/control/payload/fanout;
- generation-local ephemeral state and bounded stop/restart ownership;
- backend-owned persistent server identity and redacted secrets;
- no timing-jitter theater or public-network deanonymization testing;
- no upstream interaction.

M090 changes only the two named server-boundary details. M091, if separately unblocked, adds only earlier concurrency defense in depth and must not weaken these existing application controls.

## Containment authority

Accepted authorities remain:

- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml` plus `m061_containment.rs`;
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml` plus the M063-strengthened `m062_dependency_containment.rs`.

The M062 exact planning allowlist contains M090/M091 plan and future closure paths. This registration does not broaden production globs or authorize M091's blocked core/dependency work.

## Accepted unrelated Proposal 170 state

Tunnel-security work does not reopen the accepted RouterInfo matrix:

- 43 canonical additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable.

M051 remains blocked by absent substantive news/banned-peer owners. AddressBook and unrelated base-I2PControl limitations remain separate and must be documented truthfully.

## Registry maintenance rules

1. M090 is the sole dependency-ready tunnel-security implementation handoff.
2. M091 is registered `blocked`; its plan is not permission to alter Yosemite/core/dependencies.
3. M089 remains valid accepted reclosure evidence for `f0f3fc2204318c2fac69817d347df2702c51287b` until a later independent reclosure supersedes it for a newer head.
4. M088 remains valid evidence explaining the lower-layer gap; M091 owns any future correction rather than rewriting M088 history.
5. Preserve RouterInfo 37/1/5 and M051 unless separate source-owner work changes them.
6. Preserve ADR-0003 and the preferred `emissary-cli/src/i2pcontrol/**` production boundary wherever technically possible.
7. Unsupported/underspecified runtime options fail before allocation; persist-and-ignore is forbidden.
8. External sources remain read-only; no upstream interaction is authorized.
9. All writes remain internal to `eggstack/emissary`.
