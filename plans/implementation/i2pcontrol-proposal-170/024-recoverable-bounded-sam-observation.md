# M024 — Recoverable Bounded SAM Observation

Status: blocked

Primary class: infrastructure/capability corrective pass

Hard dependency:

- M023 closed for implementation

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Prior evidence:

- retained bounded SAM observation implementation from M016
- `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md`

## 1. Bounded objective

Correct the already-introduced read-only SAM observation path so transient capacity pressure or incomplete socket metadata does not permanently disable truthful `ClientServicesInfo.SAM` responses until router restart.

The existing architecture remains:

```text
SamServer/SamSession authoritative lifecycle
    -> bounded observation publisher
    -> read-only snapshot handle
    -> I2PControl ClientServicesInfo serializer
```

This milestone must not create a second SAM registry, control plane, session manager, polling loop, or lifecycle authority.

## 2. Current defects

The observation source has fixed session/socket bounds and one process-lifetime `overflowed` flag. Any session bound, socket bound, duplicate activation, missing peer metadata, or unknown session update can set the flag. Session/socket removal increments generation but does not clear or rebuild the incomplete state. Every subsequent SAM query may therefore fail until router restart even when the active population returns below bounds.

The per-session socket bound may be reached by ordinary streaming clients. Some socket paths record metadata before final registration and must remove it reliably on failure/close.

## 3. Required invariants

1. The SAM server/session implementation remains the sole lifecycle owner.
2. Observation is read-only outside the publisher and exposes no mutation or cancellation authority.
3. Snapshot memory is bounded by explicit session and per-session socket limits.
4. A snapshot is either complete for its advertised generation or explicitly unavailable; partial data is never returned as complete.
5. Transient overflow/incompleteness can recover without router restart once authoritative active state is representable again.
6. Duplicate or out-of-order lifecycle events cannot fabricate sessions/sockets.
7. Session removal removes every associated socket observation.
8. Socket close/failure removes the correct observation exactly once where possible.
9. Session IDs, nicknames, destination B32 addresses, socket type, and peer addresses are the only exposed fields required by the pinned contract/reference shape.
10. Private destinations, private keys, authentication material, stream payloads, and I2CP options are never exposed.
11. Snapshot acquisition does not hold a lock across await points.
12. No unbounded history, event queue, or per-request scan of unrelated router state is introduced.

## 4. Explicit non-goals

- no SAM protocol behavior change;
- no session admission-limit change unless the observation bound is demonstrably below an existing router limit and can be aligned without expanding runtime capacity;
- no user-visible SAM management method;
- no stream/datagram debugging API;
- no persistent session telemetry or historical metrics;
- no generic observer framework;
- no new dependency, CI, release, frontend, tunnel, or upstream work.

## 5. Expected file boundary

Permitted core files are only the already-modified observation path:

- `emissary-core/src/sam/mod.rs`
- `emissary-core/src/sam/session.rs`
- `emissary-core/src/sam/socket.rs`
- directly related pending connection/session files only if lifecycle ordering requires correction;
- `emissary-core/src/router/mod.rs` only for the existing handle exposure.

I2PControl files:

- `emissary-cli/src/i2pcontrol/client_services.rs`
- focused tests/docs.

Do not touch streaming algorithms, tunnel pools, NetDB, transports, proxy managers, or unrelated router modules.

## 6. Required design

### 6.1 Observation state vocabulary

Replace the single sticky boolean with an explicit bounded state such as:

- `Complete { generation, sessions }`;
- `Incomplete { generation, reason }`.

The concrete enum/name is implementation-local. Required behavior:

- entering incomplete state increments generation;
- snapshot returns a typed unavailable/incomplete error while incomplete;
- recovery creates a new complete generation only from authoritative current state or from lifecycle events proven to have restored completeness;
- old handles remain read-only views of the same source.

### 6.2 Recovery strategy

Prefer one of these bounded strategies, in order:

1. **Authoritative rebuild hook:** SamServer already owns the complete active session map and can rebuild the bounded observation snapshot synchronously when the population is within limits.
2. **Dirty-and-rebuild on lifecycle boundary:** mark dirty on an unrepresentable event and rebuild from the authoritative session context after the event completes.
3. **Deterministic local recovery:** only if all omitted entities are known and tracked without extra unbounded state, clear incomplete when the exact entities causing overflow have exited.

Do not simply clear the flag when count drops; that would return a snapshot that may still be missing sockets/sessions.

### 6.3 Bounds

- Derive observation session bound from an existing SAM admission/configuration bound when one exists; otherwise retain the current bound with documentation.
- Reassess the per-session socket bound against normal protocol behavior. Increase it only to a conservative fixed value justified by current SAM limits and response budget.
- ClientServicesInfo response-budget estimation must use the same constants.
- Bound violation fails explicitly; no silent truncation.

### 6.4 Lifecycle event coverage

Audit and prove observation transitions for:

- primary session activation;
- sub-session handling where represented by the pinned response;
- control/session socket;
- outbound stream connect pending/success/failure;
- accept/forward registration success/failure;
- active stream close/reset;
- listener replacement/close;
- session teardown and server shutdown.

Do not add observation for protocol entities not returned by Proposal 170.

### 6.5 Metadata completeness

Peer metadata absence must be handled according to the actual pinned/reference shape:

- if peer is required, the snapshot remains incomplete until a truthful value exists;
- if peer may be absent, serialize the exact permitted neutral representation;
- do not treat a harmless absent optional field as global permanent overflow.

Record the decision in tests/docs.

## 7. Failure, cancellation, restart, and contention semantics

- Publisher updates occur under the existing short synchronous lock.
- Snapshot cloning sees one complete generation or returns incomplete.
- Recovery rebuild is bounded and cannot hold the lock while awaiting network/protocol work.
- Session/task cancellation must remove or rebuild observation before the authoritative owner forgets the entity.
- Router restart begins with an empty complete generation.
- Generation wraps safely without using generation alone as security or uniqueness authority.
- Poisoned/invariant-broken state fails closed; it does not return stale complete data.

## 8. Compatibility

- Public ClientServicesInfo shape remains unchanged.
- Existing disabled/listener-unavailable behavior remains unchanged.
- Transient overflow changes from restart-sticky failure to recoverable explicit unavailability.
- No persisted data or configuration migration.
- No change to SAM client protocol behavior.

## 9. Focused tests

Required core tests:

1. Empty source returns complete empty snapshot.
2. Session activation/removal round-trip updates generation and contents.
3. Every socket type enters/exits correctly.
4. Registration failure removes provisional observation.
5. Session teardown removes all sockets.
6. Bound overflow enters incomplete state and snapshot fails.
7. Population returns within bounds; authoritative rebuild produces a complete accurate snapshot without restart.
8. Recovery does not resurrect removed sessions or omit retained sessions.
9. Duplicate/out-of-order events fail closed and recover through rebuild.
10. Concurrent snapshot/update yields coherent before/after/incomplete outcomes.
11. No private destination/key/option material occurs in snapshot debug/serialization.
12. Response-budget constants match observation bounds.

Required CLI tests:

13. Listening SAM with complete source returns exact sessions object.
14. Listening SAM with incomplete source returns explicit sanitized method failure.
15. Recovered source returns successful current sessions on a later request.
16. Disabled/stopped SAM does not require the observation handle.
17. Missing handle while listener active remains fail closed.

## 10. Verification commands

```bash
cargo test -p emissary-core sam_session_observation
cargo test -p emissary-core sam
cargo check -p emissary-core
cargo test -p emissary-core
cargo clippy -p emissary-core --all-targets -- -D warnings
cargo test -p emissary-cli --no-default-features --features i2pcontrol sam
cargo test -p emissary-cli --no-default-features --features i2pcontrol client_services
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

If full core tests encounter a documented environment file-descriptor ceiling, use the existing local `ulimit` workaround and record it. Do not build new CI around it.

## 11. Documentation and static guards

- Document observation bounds, incomplete semantics, recovery trigger, and fields exposed.
- Add a static relationship/assertion between response-budget and observation constants where practical.
- Preserve the no-lifecycle-authority statement.
- Do not claim a live-network end-to-end test unless one actually runs.

## 12. Acceptance criteria

M024 is implementation-complete only when:

- incomplete observation is explicit and recoverable without restart;
- recovery rebuild is derived from authoritative active state, not flag clearing;
- lifecycle tests cover activation, every represented socket path, failure, close, teardown, overflow, and recovery;
- ClientServicesInfo returns no partial snapshot as complete;
- bounds remain fixed and response-budget aligned;
- core changes remain restricted to the existing observation seam;
- core/CLI verification passes or exact unrelated blockers are recorded;
- no SAM protocol, streaming, tunnel, dependency, CI, or upstream expansion occurs;
- an implementation disposition records the recovery strategy and residual evidence limitations.

## 13. Stop conditions

Stop if:

- authoritative rebuild requires a broad SAM session-registry redesign;
- accurate socket metadata requires packet/payload inspection or private material exposure;
- proposed recovery uses unbounded event history;
- a higher observation bound would become a runtime admission expansion rather than an observation correction;
- work reaches streaming algorithms, missing tunnels, CI, or upstream activity.

M024 closure supplies the final SAM source evidence required by M025.