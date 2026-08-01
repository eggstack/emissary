# I2PControl Proposal 170 Milestone M024 — Closure Status

Status: closed internally against pinned revision

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/024-recoverable-bounded-sam-observation.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Repository baseline reviewed: `d73a44d`

Implementation commit:

- `d73a44d` — `fix(sam): make bounded observation recoverable`

Implementation disposition:

- `plans/closure/i2pcontrol-proposal-170/024-implementation-disposition.md`

## 1. Executive finding

M024 is complete for its bounded SAM observation corrective scope. The
process-lifetime sticky overflow flag has been replaced with explicit
complete/incomplete semantics. Exact active lifecycle records remain available
for a finite recovery window, and a later generation is published only after
close/removal events prove that the current state is within the public session,
socket, and metadata bounds. ClientServicesInfo therefore cannot serialize a
partial SAM snapshot as complete and can recover without router restart from
ordinary transient pressure.

The SAM server/session implementation remains the sole lifecycle authority.
The additional stream metadata path only transports existing observation IDs
for exact passive cleanup; it does not add a supervisor, registry, protocol
method, or data-plane behavior.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Empty source is complete | `observation_starts_empty` | pass |
| Activation/removal updates contents and generation | `cloned_handles_read_the_same_state`, `session_removal_is_visible`, generation assertions | pass |
| Every represented socket type is published | Existing activation/stream/accept/forward publisher call sites; serializer fixtures; exact socket event IDs | pass |
| Registration failure removes provisional state | `on_stream_accept`/`on_stream_forward` remove the provisional ID on registration error; publisher recovery tests cover matching removal | pass |
| Session teardown removes all sockets | `SamServer` removes the session; publisher `remove_session` drops the full session and unknown socket debt | pass |
| Bound overflow is explicit | `per_session_socket_bound_is_explicit`, `session_bound_is_explicit` | pass |
| Session-bound recovery is accurate | `session_bound_is_explicit` removes a retained session and verifies the overflow session is present in the rebuilt 1000-session snapshot | pass |
| Socket-bound recovery is accurate | `per_session_socket_bound_is_explicit` removes one exact socket and verifies the retained overflow socket is included | pass |
| Missing metadata recovers only after proof | `missing_peer_fails_closed` remains unavailable until session removal, then returns complete empty state | pass |
| Unknown/out-of-order updates fail closed and recover | `unknown_socket_update_recovers_after_matching_close`, `duplicate_activation_fails_closed_without_fabricating_state` | pass |
| Exact active stream close/reset cleanup | `StreamClosed`/`StreamRejected` carry the observation ID for connect/accept sockets; forwarded listeners retain `None` and are removed at session teardown | pass |
| Coherent snapshots under contention | Existing short-lock snapshot design and generation-based publisher updates; no await is performed under the lock | pass |
| No private material | Snapshot state contains only the pinned public fields; existing CLI serialization test `serialize_sam_sessions_preserves_pinned_active_shape` remains passing | pass |
| Response budget alignment | CLI budget calculation imports the same core bounds; focused client-services and static guard suites pass | pass |

## 3. Verification executed

Commands and outcomes:

```text
cargo test -p emissary-core observation_tests
  13 passed
cargo test -p emissary-core sam
  162 passed
cargo check -p emissary-core
  passed
cargo test -p emissary-core
  1066 passed, 2 ignored
cargo clippy -p emissary-core --all-targets -- -D warnings
  passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol client_services
  86 passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol sam
  27 passed
cargo check -p emissary-cli --no-default-features --features i2pcontrol
  passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol
  1188 passed
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
  passed
cargo +nightly fmt --manifest-path emissary-core/Cargo.toml
cargo +nightly fmt --manifest-path emissary-cli/Cargo.toml
  passed
```

The workspace-wide stable formatter check reports the repository's existing
stable/nightly rustfmt configuration mismatch. The touched manifests were
formatted with nightly; no unrelated tutorial or workspace formatting change
was made.

## 4. Invariant review

1. SAM lifecycle ownership remains in `SamServer` and `SamSession`.
2. The administrative handle exposes no mutation, cancellation, or task
   control authority.
3. Public snapshot memory remains bounded by 1000 sessions and 8 sockets per
   session; finite recovery storage is separately capped at 2x those values.
4. Only complete generations can be cloned; incomplete state fails closed.
5. Duplicate, unknown, missing-peer, and out-of-order transitions cannot
   fabricate a session or socket.
6. Session removal and exact socket close/failure paths remove associated
   observations once where the lifecycle supplies an ID.
7. Generation wrapping remains safe because generation is diagnostic/publication
   state, not an identity or security authority.
8. No unbounded history, polling loop, generic observer, or request-time scan
   of unrelated router state was added.

## 5. Compatibility, migration, and security review

No SAM protocol behavior, public ClientServicesInfo shape, persisted data, or
configuration format changed. Disabled and listener-unavailable behavior is
unchanged. Incomplete SAM observation now returns a sanitized unavailable
method error rather than the old restart-sticky overflow result. No secret,
private destination, key, credential, option, or payload is included in a
snapshot, debug representation, serializer, or error.

## 6. Unresolved findings and successor state

| Severity | Finding | Disposition |
|---|---|---|
| none within M024 | Sticky SAM observation overflow | resolved by this closure |
| medium evidence limitation | No live-network SAM activation harness | retained as a qualified limitation; deterministic seam evidence is sufficient for this bounded internal closure |
| high claim defect | RouterInfo selector/source contradictions | remains M025–M027 scope |

M025 is now unblocked and marked `ready` because M020, M022, M023, and M024
are closed. M026 remains blocked on M025. M027 remains blocked on M020–M026.
The subsystem remains `corrective pass required`; only M027 may restore a final
subsystem disposition.

## 7. Planning and external boundary

Updated in the closure/planning commit following this implementation:

- M024 implementation plan: `implemented`;
- M024 closure and implementation disposition: this record and its companion;
- registry and implementation index: M025 is the dependency-ready handoff;
- subsystem roadmap and support status: M024 closed, M025 ready.

All external sources were accessed read-only. No upstream repository or
maintainer channel was mutated, and no upstream review, merge, adoption,
submission, or contribution artifact was requested or prepared.

## 8. Bounded closure statement

M024 is closed internally against the pinned Proposal 170 revision. Its
recoverable bounded SAM observation source is implemented and verified without
expanding SAM protocol behavior or router lifecycle authority. The remaining
Proposal 170 work is correctly handed to M025.
