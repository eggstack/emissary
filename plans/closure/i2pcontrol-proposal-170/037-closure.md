# I2PControl Proposal 170 Milestone M037 — Closure Status

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/037-containment-boundary-reduction.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md#6-milestones`

Repository baseline reviewed: `5afe953`

Implementation commit:

- `5e003fe` — `feat: reduce I2PControl containment coupling`

Frozen implementation/test head reviewed: `5e003fe`

Review date: 2026-08-05

## 1. Executive finding

M037 is complete. Proposal 170 AddressBook administrative ownership,
persistence/migration/repair policy, bounded subscription control, and SAM
observation aggregation/recovery now live under `emissary-cli/src/i2pcontrol/`.
The original AddressBook and core SAM files retain only typed runtime adapters,
composition seams, and the minimum optional passive SAM hook required to see
authoritative lifecycle order. The Proposal 170 wire contract, persisted
formats, disabled mode, and runtime support classification are unchanged.

The hook is absent by default, publishes sanitized lifecycle facts
synchronously, carries no socket/session/key/control handles, and reports
publication failure without changing SAM lifecycle behavior. I2PControl owns
bounded maps, overflow/recovery policy, incomplete-snapshot rejection, and
serialization.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| Wire and runtime behavior remain stable | no-feature and feature-enabled package suites; AddressBook tests | pass | No Proposal 170 method, response, status, or persistence format changed. |
| Disabled/default AddressBook isolation remains exact | no-feature check and full no-feature CLI suite | pass | The new owner is feature-gated and legacy behavior remains available. |
| Base32/Base64/full-destination ownership remains coherent | AddressBook focused suite and runtime-owner extraction review | pass | Existing resolution and validation helpers retain their prior behavior. |
| Active subscription semantics remain bounded and truthful | no-feature AddressBook suite and feature-enabled package suite | pass | The bounded command seam remains the runtime adapter. |
| SAM complete snapshots retain prior semantics | `sam_observer` lifecycle fixture test | pass | Complete bounded state serializes through the existing ClientServices path. |
| SAM incomplete state recovers authoritatively | `incomplete_state_recovers_after_authoritative_removal` | pass | Removal events recover state; partial snapshots fail closed. |
| Optional observer is absent without core state | `absent_observer_has_no_core_state`; core SAM suite | pass | No observer means no aggregation maps or queue in core. |
| Hook metadata is sanitized and handle-free | static containment test over `SamObservationEvent` | pass | No sockets, destinations, keys, channels, or mutable session state cross. |
| Bounds, overflow, ordering, and failure are I2PControl-owned | `sam_observer.rs` bounded maps and direct synchronous hook | pass | No await, polling loop, global bus, or unbounded queue was introduced. |
| Changed paths and dependencies are guarded | `m037_containment` manifest/import tests | pass | Production paths are classified and prohibited imports/allocations are checked. |
| Unsupported tunnel backends stay resource-free | static unsupported-backend guard | pass | No lifecycle or data-plane capability was added. |

## 3. Verification executed

### Commands run

```bash
cargo check -p emissary-core
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-core sam --no-fail-fast
cargo test -p emissary-core --no-fail-fast
cargo test -p emissary-cli --no-default-features address_book --no-fail-fast
cargo test -p emissary-cli --no-default-features --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol sam_observer --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m037_containment --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo clippy -p emissary-core --all-targets -- -D warnings
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

### Results

- Core and CLI feature/no-feature checks: pass.
- Core SAM and full core suites: pass.
- No-feature AddressBook and full CLI suite: pass.
- Feature-enabled SAM observer, containment, and full CLI suites: pass.
- Core and I2PControl clippy with `-D warnings`: pass.
- `git diff --check`: pass.
- `cargo fmt --all -- --check`: not pass on the repository baseline; the
  available stable formatter reports the checked-in nightly/stable option
  mismatch across unrelated pre-existing files. Targeted changed files were
  formatted and no formatter spillover was retained.

## 4. Invariant, failure, and security review

The passive hook performs no await or blocking operation in the SAM poll path.
The I2PControl observer uses bounded `BTreeMap` state, fixed per-session socket
limits, generation bookkeeping, and explicit recovery/fail-closed behavior.
Publication failure is returned to the lifecycle caller and does not tear down
or mutate SAM ownership. The default path has no observer and therefore no
additional state or work.

Only sanitized identifiers, socket metadata, and bounded peer text cross the
core seam. No private key, destination object, live socket, runtime command
channel, or mutable session handle is exposed. Unsupported tunnel backends
remain explicit and allocate no listener/task resource.

## 5. Compatibility and migration review

No migration is required. Existing AddressBook control-state, subscription,
legacy destination, tunnel-definition, and server-identity formats remain
readable. No I2PControl wire method or response shape changed. Legacy
downloader/runtime ownership remains in the original module; the new
administrative owner consumes validated runtime facts through the existing
bounded seam.

## 6. Documentation and roadmap disposition

The implementation disposition records the changed-path classification,
before/after policy-file counts, and the reason for every retained
non-I2PControl block. The registry, subsystem roadmap, M037 plan, and handoff
README now record M037 closed. M038 is unblocked and ready; M039 remains
blocked on M038 as required by the planning process.

No high or medium M037 containment, compatibility, security, or behavior
finding remains. The stable/nightly formatter mismatch is retained as a low
baseline tooling finding.

## 7. Internal-only attestation

No upstream repository, issue, pull request, review, submission, maintainer
channel, or third-party connector was mutated. The requested commit and push
apply only to the internal repository branch.
