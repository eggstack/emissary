# M040 — Startup Server Cancellation-Owner Correction

Status: ready

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Corrective authority:

- `plans/closure/i2pcontrol-proposal-170/039-closure-invalidation.md`

Applicable governance and decisions:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`
- M032 implementation and closure records

Repository baseline:

- `563e093ba1e65b4edc31104e3045c8b5a665e8ed`

## 1. Bounded objective

Restore existing startup-managed generic server tunnel behavior after the M032
runtime extraction introduced an immediately closed cancellation channel.

The startup `ServerTunnelManager` must retain the cancellation sender for the
complete lifetime of each `run_single_server` invocation so the reusable runtime
does not interpret channel closure as an immediate stop request.

M040 also adds direct regression evidence against the original startup manager
path. It does not change control-plane ownership, add administrative control of
startup tasks, modify the generic server backend contract, or implement another
tunnel family.

## 2. Demonstrated defect and why prior verification missed it

Current startup code creates the runtime channel as:

```rust
let (_, cancellation) = tokio::sync::watch::channel(false);
```

The sender is dropped before the runtime begins. `watch::Receiver::changed()`
then completes because the channel is closed, and `run_single_server` may exit
before SAM session creation or forwarding.

M032 tests exercise `run_single_server` and the control-plane server backend with
a retained sender. M038 configures a startup client but not a startup server.
No accepted test runs `ServerTunnelManager` through session creation and
`STREAM FORWARD`, so the startup-only lifetime defect escaped.

## 3. Required invariants

1. Existing startup server definitions remain startup-owned and read-only.
2. The startup manager retains one cancellation sender while its matching
   runtime is alive.
3. The retained sender is not exposed through I2PControl and does not create an
   administrative stop handle.
4. Control-plane server supervisor cancellation remains unchanged.
5. Destination generation/loading, destination observation, SAM options,
   forwarding, and retry behavior remain unchanged except for removal of the
   accidental immediate cancellation.
6. No startup server task is adopted by the I2PControl supervisor.
7. No `emissary-core/**` production change.
8. No new public protocol field, method, status, alias, or tunnel type.
9. No new data-plane implementation.
10. No upstream interaction.

## 4. Scope and production file budget

### Authorized production file

- `emissary-cli/src/tunnel/server.rs`

The production diff should be limited to cancellation-owner lifetime and any
small private helper required to make the lifetime explicit and testable.

### Authorized tests and records

- focused unit tests in `emissary-cli/src/tunnel/server.rs`, or one narrowly
  named integration test under `emissary-cli/tests/` if the private manager path
  cannot be exercised adequately in-module;
- M040 implementation disposition and closure record;
- directly affected tunnel documentation only if current wording describes the
  broken behavior.

### Prohibited production changes

- `emissary-core/**`;
- `emissary-cli/src/i2pcontrol/backends/server.rs` unless a test-only seam is
  strictly required and no behavior changes;
- TunnelManager wire/domain/persistence changes;
- startup inventory ownership changes;
- client tunnel changes;
- HTTP/SOCKS/IRC/CONNECT/Streamr/bidirectional tunnel work;
- general cancellation frameworks or task registries;
- CI, release, packaging, frontend, or unrelated cleanup.

## 5. Required production change

The startup server event loop must retain the watch sender in scope until
`run_single_server` returns. A minimal shape is acceptable:

```rust
let (_cancellation_keepalive, cancellation) =
    tokio::sync::watch::channel(false);
```

The final implementation may use a named private holder if that improves
clarity, but it must not add an external stop/control surface.

The implementation must inspect all other `run_single_server` call sites and
prove that each one has intentional sender ownership:

- control-plane server backend: named supervisor owns the sender;
- startup manager: local keepalive owns the sender;
- tests: sender lifetime is explicit.

## 6. Ordered work packages

### WP1 — Add the failing startup-manager regression

Construct a bounded fake SAM listener that records:

- `HELLO` negotiation;
- `SESSION CREATE`;
- the destination returned to the runtime;
- `STREAM FORWARD`.

Run the original startup manager or its exact private server event loop. Before
the fix, the test must fail because the runtime exits before completing the
expected SAM sequence.

The test must not replace `ServerTunnelManager` with the control-plane backend.

### WP2 — Correct sender ownership

Retain the sender for the runtime lifetime with the smallest possible change.
Do not add an I2PControl handle, shared map, cancellation registry, or startup
administrative API.

### WP3 — Verify startup behavior and isolation

Prove:

- destination observation fires with the fake SAM destination;
- `STREAM FORWARD` is reached;
- the task remains alive after readiness rather than exiting from channel
  closure;
- aborting the test task performs bounded cleanup;
- an invalid/session-failing fake SAM path remains isolated and does not panic;
- control-plane server backend tests remain unchanged and pass.

### WP4 — Record changed-path justification

The implementation disposition must explain why the one production change
outside `i2pcontrol/**` is required to preserve pre-existing startup behavior.

## 7. Failure, cancellation, restart, and contention semantics

- Startup server tasks continue to have no administrative cancellation API.
- The local sender exists only to keep the watch channel open.
- Process/task shutdown remains the owner of startup task termination.
- SAM/session/forward failures retain the existing bounded retry/error behavior.
- No lock, store, or runtime map is introduced.
- Multiple startup server definitions remain independent.
- Control-plane start/stop/restart semantics are not modified.

## 8. Compatibility and migration

- No configuration format change.
- No persistence migration.
- No wire/API change.
- No destination identity migration.
- Existing startup server definitions regain intended behavior.

## 9. Required tests

Focused tests must include:

1. startup server reaches `SESSION CREATE` and `STREAM FORWARD`;
2. destination observer receives the public destination;
3. runtime remains alive after readiness while the sender is retained;
4. sender closure in a dedicated low-level fixture still cancels
   `run_single_server`, proving the reusable primitive's contract remains
   intentional;
5. control-plane server lifecycle tests still pass;
6. no-feature startup-server test path passes where applicable.

The regression must fail on baseline `563e093` and pass after correction.

## 10. Verification commands

At minimum:

```bash
cargo test -p emissary-cli --no-default-features tunnel::server
cargo test -p emissary-cli --no-default-features --features i2pcontrol tunnel::server
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

Use targeted formatting. Do not expand CI or add network-dependent tests.

## 11. Documentation and static guards

- Add an implementation disposition and independent closure record.
- Update the active registry only after closure evidence is accepted.
- Preserve the M039 invalidation until M044 reclosure.
- If a static guard is added, it must be narrowly scoped to preventing a
  sender-discard pattern at the startup call site; do not add a general source
  scanner.

## 12. Acceptance criteria

M040 may close only when:

- the startup manager retains the sender for the complete runtime lifetime;
- the direct startup-manager regression reaches session creation, destination
  observation, and forwarding;
- the test would have caught the original defect;
- existing control-plane server behavior remains unchanged;
- no core, protocol, ownership, or tunnel-family expansion occurred;
- no unresolved high- or medium-severity finding remains in this slice;
- exact verification results and changed paths are recorded.

## 13. Stop conditions

Stop and report blocked rather than:

- modify core;
- expose startup-task cancellation to I2PControl;
- refactor both tunnel managers into a general supervisor;
- alter destination storage or server backend ownership;
- implement a missing tunnel type;
- add CI/release infrastructure;
- interact with upstream.

## 14. Closure evidence required

The M040 disposition and closure must contain:

- implementation/test commit SHA;
- failing-before/passing-after regression description;
- exact fake SAM transcript or asserted command sequence;
- verification command outcomes;
- changed-path classification;
- confirmation that startup ownership and control-plane ownership remain
  separate;
- unresolved findings with severity;
- internal-only/no-upstream attestation.