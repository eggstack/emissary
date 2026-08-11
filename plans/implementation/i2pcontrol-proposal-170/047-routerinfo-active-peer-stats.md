# M047 — RouterInfo Active-Peer Statistics

Status: closed

Planning baseline: `b759038`

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Hard dependency: M046 closed

Milestone class: capability + containment invariant

## 1. Objective

Implement `i2p.router.netdb.activepeers.stats` from bounded current transport facts while preserving the audited transport data plane.

The existing I2PControl `ActivePeerStats` DTO/serializer remains the wire owner. Core may expose only neutral peer-observation fields that have canonical current owners.

## 2. Readiness audit required before editing

Inspect the exact Proposal 170 object shape and the NTCP2/SSU2 session state already maintained by Emissary. For every required field, identify the authoritative owner and update point. Classify each as directly available, cheaply countable at an existing transition, or absent.

Do not infer latency, direction, byte counts, state, IP/port, version, or capabilities from semantically adjacent data. If a required field has no canonical source, add only a neutral passive counter/value at the existing ownership point when doing so does not affect protocol behavior.

## 3. Invariants

1. No socket, cryptographic state, session key, Noise state, channel, mutable connection object, or transport command handle crosses the inspection boundary.
2. Observation does not alter scheduling, timeouts, congestion, handshake, queueing, disconnect, or retransmission behavior.
3. Per-peer memory is bounded by existing active transport sessions; snapshots impose an explicit response cap.
4. I2PControl owns wire labels/types, redaction, ordering, aggregate bounds, and error mapping.
5. Unsupported/missing individual facts are not serialized as plausible defaults merely to complete the object.

## 4. Production budget

I2PControl paths as required plus the M046 neutral inspection seam. Additional core exceptions are permitted only when the readiness audit proves necessity:

- `emissary-core/src/transport/ntcp2/**` for passive NTCP2 facts;
- `emissary-core/src/transport/ssu2/**` for passive SSU2 facts.

No crypto, I2NP, NetDB, tunnel, router algorithm, or protocol-message change is authorized.

## 5. Work packages

1. Pin an exact field-to-owner matrix against Proposal 170/reference behavior.
2. Extend the neutral transport snapshot with the minimum sanitized peer DTO.
3. Populate directly available fields at snapshot time; add passive counters only at already-existing transitions where necessary.
4. Map neutral DTOs to `ActivePeerStats` in `i2pcontrol::production` and serialize via existing handler code.
5. Add bounds, deterministic ordering, session-churn tests, transport-type tests, and secret/static guards.
6. Change the single contract row to available only after all mandatory object fields are truthful.

## 6. Failure, cancellation, restart, contention

Snapshot acquisition must never await while holding transport locks. A peer disappearing during collection is omitted from that completed snapshot rather than represented with partial invented data. Observation state is process-local and reconstructed from active sessions; no migration or persistence is introduced.

## 7. Verification

Run focused NTCP2/SSU2 tests for touched observation points, I2PControl RouterInfo fixtures, feature/no-feature CLI suites, core package tests, clippy for changed packages, static secret/handle guards, and `git diff --check`.

## 8. Acceptance criteria

The selector returns exact bounded objects for actual active peers; all mandatory fields have named owners; empty output means an authoritative snapshot with zero peers; transport behavior is unchanged; no sensitive/live handle crosses the seam; changed core lines are observation-only and independently reviewed.

## 9. Stop conditions

Stop if completing the object requires new network probes, latency measurement traffic, transport policy changes, unbounded history, or exposure of live session state.

## 10. Closure evidence

Closure must include the field-owner matrix, exact wire fixture, active-session lifecycle tests, resource bound evidence, changed-path containment review, feature-isolation evidence, and internal-only attestation.

Closure record: `plans/closure/i2pcontrol-proposal-170/047-closure.md`.
