# M024 Implementation Disposition — Recoverable Bounded SAM Observation

Status: implemented

Implementation commit: `d73a44d` — `fix(sam): make bounded observation recoverable`

Recovery strategy: deterministic bounded local recovery from exact lifecycle
records. The publisher retains a finite recovery window for sessions and
sockets that are active but temporarily outside the public response bound or
missing required peer metadata. A complete generation is rebuilt only after
the close/removal events prove that the tracked current state is again fully
representable. The implementation never clears an incomplete flag while an
unknown entity, missing peer, or bound violation remains.

## Requirement evidence

| Requirement | Evidence |
|---|---|
| Explicit complete/incomplete state | `SamSessionObservationPhase` replaces the process-lifetime `overflowed` bit; snapshots return `Incomplete` while unsafe. |
| Bounded recoverable state | Public session/socket bounds remain 1000/8; finite recovery capacity is 2x each bound and has no event queue or polling loop. |
| Accurate recovery | Recovery tests cover missing-peer teardown, socket overflow, session overflow with retained overflow state, unknown socket close, and duplicate activation. |
| Lifecycle ownership | `SamServer` remains the only session lifecycle owner; the handle is read-only outside the publisher. |
| Exact socket removal | Existing SAM observation IDs are carried through connect/accept stream close/rejection events; forwarded TCP listeners use no socket ID and remain observed until their own session teardown. |
| Failure and cancellation cleanup | Registration failure removes the provisional socket; session termination removes the complete session and all associated socket records; duplicate/out-of-order events fail closed. |
| Response bounds | `ClientServicesInfo` continues to use `SAM_SESSION_OBSERVATION_LIMIT`, `SAM_SOCKET_OBSERVATION_LIMIT`, and the 1 MiB estimate. No truncation was added. |
| Data minimization | Snapshots expose only session ID, nickname, destination B32 address, socket type, and peer address. Private destinations, keys, options, and payloads are not copied or serialized. |
| Contention semantics | Snapshot clones under a short read lock and never awaits while holding it; generation changes distinguish before/after publications. |

## Exact changed files

- `emissary-core/src/sam/mod.rs`
- `emissary-core/src/sam/session.rs`
- `emissary-core/src/sam/protocol/streaming/mod.rs`
- `emissary-core/src/sam/protocol/streaming/listener.rs`
- `emissary-cli/src/i2pcontrol/client_services.rs`
- `docs/i2pcontrol/client-services.md`
- `docs/i2pcontrol/proposal-170-support.md`
- M024 planning, registry, and roadmap status records

The streaming changes are limited to carrying already-existing SAM socket
observation IDs through internal stream lifecycle events. They add no SAM
protocol field, data-plane behavior, listener authority, or new task.

## Compatibility and security

The public `ClientServicesInfo` shape and SAM client protocol are unchanged.
Transient incompleteness is an explicit sanitized method failure rather than a
partial success. Existing disabled/stopped behavior remains unchanged. No
private material, credentials, stream payload, or I2CP option is included in
the observation state or errors.

## Residual evidence

- No live-network SAM activation test is claimed; the publisher and production
  serializer seams are covered by deterministic tests.
- If the finite recovery window itself is exceeded, the source remains
  unavailable until a new SAM server instance is created. This is the bounded
  fail-closed behavior for an active population that is not representable by
  the configured observation limits; it is not a restart requirement for
  ordinary transient pressure.
- Final Proposal 170 selector/source reconciliation remains M025–M027 scope.

## Scope and external-interaction attestation

All external specification/reference material was used read-only. No upstream
repository, issue, pull request, review, maintainer channel, or external write
was used or prepared. No tunnel data plane, router algorithm, dependency, CI,
frontend, or upstream contribution work was added.

Frozen implementation/test head: `d73a44d`.
