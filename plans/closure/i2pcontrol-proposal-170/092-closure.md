# M092 Closure — M091 Authorization, Dependency, and Containment Corrective

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/092-m091-authorization-and-containment-corrective.md`.

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Corrective predecessors:

- M090 server loopback and IRC half-close corrective and `plans/closure/i2pcontrol-proposal-170/090-closure.md`;
- M091 pre-accept stream concurrency plan and `plans/closure/i2pcontrol-proposal-170/091-closure.md`;
- `plans/003-planning-process.md` §§6–8 and §11.

Planning baseline: `944da7b887b6efbd46601e9fad1c853581f40b8e`.

Known valid pre-M091 implementation/closure baseline: `6d631d4423c7faa761b47a84e07436bbaf5d9ad4`.

Implementation head: `8860407a79347ce925603821cdb231e47a680623`.

Review date: 2026-08-27.

## 1. Disposition

M092 is complete. The M091 production/dependency/vendor delta that landed at `5053ce6b595351b251afb36f1f7d5278ef8f58d1` while its registered plan was `blocked` has been removed. Crates.io Yosemite `0.7.0` is again the sole source for the `yosemite` dependency; `vendor/yosemite/**` is gone; the three `emissary-core` SAM/streaming changes are reverted; the accepted-server lower-layer option seam is removed; and M060/M061/M062 production/dependency containment semantics are restored to their pre-M091 authority.

M090's resolver-free server targets and IRC half-close/drain behavior are intact and untouched. The M088 pre-accept / lower-layer resource/timing residual is again the current accepted disposition. M091 is truthfully represented as `blocked / superseded by M092`; its technical implementation and test evidence are retained in `plans/closure/i2pcontrol-proposal-170/091-closure.md` and re-marked as `corrective pass required / superseded by M092`, but the closure record is not current authority and did not retroactively authorize the deprecated vendored-Yosemite dependency strategy.

M092 does not introduce any new production behavior, alternative lower-layer transport, vendoring strategy, parallel SAM stack, broader `emissary-core` hook, or replacement dependency mechanism.

## 2. Changed/deleted-path matrix

Working-tree diff `6d631d4423c7faa761b47a84e07436bbaf5d9ad4..<implementation head>`:

| Path | Change | Authority |
|---|---|---|
| `Cargo.toml` | Modified to restore crates.io Yosemite 0.7.0 (removed `path = "vendor/yosemite"`) | M092 §5 |
| `Cargo.lock` | Modified to restore registry source and checksum on the `yosemite` package; `smol`/`tracing-subscriber` removed from yosemite deps | M092 §5 |
| `emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs` | Modified to remove `max_concurrent_streams` / `NonZeroUsize` / mutex imports and the concurrency-capture test added by M091 | M092 §5 |
| `emissary-core/src/sam/protocol/streaming/config.rs` | Modified to remove `MAX_CONCURRENT_STREAMS_OPTION`, `MAX_CONFIGURED_CONCURRENT_STREAMS`, `from_session_options`, and the `hashbrown::HashMap` import | M092 §5 |
| `emissary-core/src/sam/protocol/streaming/mod.rs` | Modified to remove `pub(crate) mod config;`, `new_with_session_options`, `max_concurrent_streams` field, `PENDING_STREAM_PRUNE_THRESHOLD` extra capacity logic, and the four M091 concurrency regression tests; restored M090 closure state | M092 §5 |
| `emissary-core/src/sam/session.rs` | Modified to remove `use crate::sam::protocol::streaming::config;` and the `new_with_session_options` call wired by M091 | M092 §5 |
| `vendor/yosemite/Cargo.toml` | Deleted | M092 §5 |
| `vendor/yosemite/LICENSE` | Deleted | M092 §5 |
| `vendor/yosemite/README.md` | Deleted | M092 §5 |
| `vendor/yosemite/examples/*` (10 files) | Deleted | M092 §5 |
| `vendor/yosemite/src/**` (24 files) | Deleted | M092 §5 |
| `emissary-cli/tests/m060_containment.rs` | Modified to remove `emissary-core/src/sam/protocol/streaming/config.rs` from the production allowlist and the M091 comment | M092 §5, §6 |
| `emissary-cli/tests/m062_dependency_containment.rs` | Modified to restore byte-identical lockfile assertion, restore M061 source-boundary unchanged assertion, remove vendor/yosemite and emissary-core streaming paths from `is_authorized_tunnel_runtime_path`, and add exact entries for M092 and M093 planning paths to `is_authorized_planning_path` | M092 §5, §6 |
| `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml` | Modified to remove `emissary-core/src/sam/protocol/streaming/config.rs` from `core_owner_hooks` and remove the M091 evidence entry | M092 §5, §6 |
| `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml` | Modified to restore `lockfile.expected = "byte-identical to baseline"` and remove `Cargo.lock` from `allowed_production_paths.root_manifests` | M092 §5, §6 |

Planning/status bookkeeping (within the M092 section 5 "may additionally update" allowance):

| Path | Change |
|---|---|
| `plans/implementation/i2pcontrol-proposal-170/091-pre-accept-stream-concurrency-boundary-hardening.md` | Restored to the `blocked` truth at planning commit `7194fa50ac03b44fb4c08a4d4d05d5fd33ea49b3`; added `superseded by M092` status and disposition header |
| `plans/closure/i2pcontrol-proposal-170/091-closure.md` | Disposition amended to `corrective pass required / superseded by M092` with a preservation note for the prior technical evidence |
| `plans/implementation/i2pcontrol-proposal-170/092-m091-authorization-and-containment-corrective.md` | Plan file retained at its baseline content (also written during earlier planning sequence) |
| `plans/implementation/i2pcontrol-proposal-170/093-post-m092-tunnel-security-reclosure.md` | Status promoted from `blocked` to `ready` after M092 closure; §2 readiness blocker rewritten to reflect the accepted M092 closure |
| `plans/implementation/i2pcontrol-proposal-170/README.md` | Status line, current-handoff summary, historical table, corrective-sequence diagram, and final status rule updated to reflect M092 closed / M093 ready |
| `plans/registry.md` | Status line, current-tunnel-security sequence diagram, current ready-handoff section, recently-closed authority table, and registry maintenance rules updated to reflect M092 closed / M093 ready |
| `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | Status line, §1 disposition, §7 dependency graph, and §8 milestone summary updated to reflect M092 closed / M093 ready |
| `plans/closure/i2pcontrol-proposal-170/092-closure.md` | This record |

Verification (working-tree evidence review):

- `git diff --name-status 6d631d4423c7faa761b47a84e07436bbaf5d9ad4..HEAD -- Cargo.toml Cargo.lock emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs emissary-core/src/sam/protocol/streaming/config.rs emissary-core/src/sam/protocol/streaming/mod.rs emissary-core/src/sam/session.rs emissary-cli/tests/m060_containment.rs emissary-cli/tests/m062_dependency_containment.rs plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml` → all entries restored to baseline or carry only the M092-allowed exact planning-path bookkeeping; production/dependency diff is empty.
- `git ls-files | grep '^vendor/yosemite/' || true` → no entries (`vendor/yosemite/**` removed in full).
- M062 test allowlist retains the pre-M091 semantic assertions and adds only the exact M092 plan, M093 plan, and M092 closure planning paths (no globs, no production exceptions).

## 3. Proof M090 remains present

M090 production commit `172a4e86d0d183c028244b02e91440ac36525c0c` introduced:

- `emissary-cli/src/i2pcontrol/backends/http_server.rs` literal-loopback target typing;
- `emissary-cli/src/i2pcontrol/backends/http_bidir.rs` shared typed seam;
- `emissary-cli/src/i2pcontrol/backends/irc_server.rs` literal-loopback target and half-close drain.

`git diff 6d631d4423c7faa761b47a84e07436bbaf5d9ad4..HEAD -- emissary-cli/src/i2pcontrol/backends/http_server.rs emissary-cli/src/i2pcontrol/backends/http_bidir.rs emissary-cli/src/i2pcontrol/backends/irc_server.rs` → empty. M090's HTTP/IRC production delta is preserved byte-for-byte.

`cargo test -p emissary-cli --no-default-features --features i2pcontrol http_server::tests` and `irc_server::tests` remain in the green run of the I2PControl CLI suite below.

## 4. Proof M091 production/dependency files were restored to the valid pre-M091 state

For each M091-touched production/dependency file, the working-tree content is byte-identical to `6d631d4423c7faa761b47a84e07436bbaf5d9ad4:<path>`:

- `Cargo.toml`: diff stat line count against baseline is 0.
- `Cargo.lock`: diff stat line count against baseline is 0; `yosemite` 0.7.0 entry has `source = "registry+https://github.com/rust-lang/crates.io-index"` and `checksum = "c6bf3692263d7a9258016f5468c5cf5301b06189d7bc4c97b014b69022659871"` and no `smol`/`tracing-subscriber` dependency entries.
- `emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs`: diff against baseline is 0; no `max_concurrent_streams`, no `NonZeroUsize` import, no `Mutex` M091 test capture.
- `emissary-core/src/sam/protocol/streaming/config.rs`: diff against baseline is 0; no `MAX_CONCURRENT_STREAMS_OPTION`, no `MAX_CONFIGURED_CONCURRENT_STREAMS`, no `from_session_options`, no `hashbrown::HashMap` import.
- `emissary-core/src/sam/protocol/streaming/mod.rs`: diff against baseline is 0; no `pub(crate) mod config;`, no `new_with_session_options`, no `max_concurrent_streams` field, no M091 concurrency regression tests.
- `emissary-core/src/sam/session.rs`: diff against baseline is 0; no `use crate::sam::protocol::streaming::config;`, no `new_with_session_options` call.

## 5. Containment guard before/after evidence

M060 core-observation containment test (`emissary-cli/tests/m060_containment.rs`):

- Restored to M090 closure state; the `emissary-core/src/sam/protocol/streaming/config.rs` entry added by M091 is removed; the `M091-approved lower-layer streaming configuration seam` comment is removed.
- Passes (`cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m060_containment`).

M061 source-boundary assertion:

- Restored to M090 semantic state via `m062_dependency_containment.rs::m061_source_boundary_files_remain_unchanged`, which now re-enforces that the retained M061 source-boundary files are unchanged relative to the M062 baseline.

M062 dependency-containment test (`emissary-cli/tests/m062_dependency_containment.rs`):

- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment` → all 19 tests pass.
- `lockfile_is_byte_identical_to_fork_baseline` again asserts byte-identical Cargo.lock against `fork_baseline`.
- `m061_source_boundary_files_remain_unchanged` again enforces the M061 source-boundary integrity.
- `allowed_production_paths_match_the_m062_budget` is exercised with the diff against `fork_baseline`; only the M092 plan, M093 plan, M092 closure, and other pre-existing planning allowlist entries appear. The exact planning-path entries for `092-m091-authorization-and-containment-corrective.md`, `093-post-m092-tunnel-security-reclosure.md` were added to `is_authorized_planning_path` per M092 §6.

`plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml`:

- Restored to M090 closure state; the `emissary-core/src/sam/protocol/streaming/config.rs` `core_owner_hooks` entry and its M091 evidence block are removed.

`plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`:

- Restored to M090 closure state; `lockfile.expected` is `byte-identical to baseline`; the M091 `Cargo.lock` allowlist entry is removed.

## 6. M091 plan/closure disposition evidence

`plans/implementation/i2pcontrol-proposal-170/091-pre-accept-stream-concurrency-boundary-hardening.md`:

- Status: restored to `blocked / superseded by M092`.
- Body: restored to the pre-implementation truth at planning commit `7194fa50ac03b44fb4c08a4d4d05d5fd33ea49b3`.
- A `Disposition:` header was prepended stating that the plan is corrective-pass-required and that the technical implementation/test evidence remains in `plans/closure/i2pcontrol-proposal-170/091-closure.md` but is not current authority.

`plans/closure/i2pcontrol-proposal-170/091-closure.md`:

- Disposition amended to `corrective pass required / superseded by M092`.
- A historical-disposition note was prepended stating that the technical evidence is retained but not current authority because the dependency strategy was not authorized before implementation.
- The technical test evidence (vendored-Yosemite dependency declaration, requirement-to-evidence matrix, containment summary, verification command outcomes, M091 bounded post-accept admission defense in depth, M091 lower-layer check ordering, M091 per-peer/rate residual disposition) is not rewritten as if it never occurred; it is preserved as historical evidence and explicitly disclaimed as current authority.

## 7. Verification command outcomes

| Command | Outcome |
|---|---|
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass; no errors |
| `cargo test -p emissary-core` | pass; 1062 tests, 2 ignored |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | pass; 1696 tests across 24 suites |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m060_containment --test m061_containment --test m062_dependency_containment` | pass; 29 tests across 3 suites |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment` | pass; 19 tests |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass; no issues |
| `git diff --check` | pass |
| `git ls-files | grep '^vendor/yosemite/'` | empty |
| `git diff 6d631d4423c7faa761b47a84e07436bbaf5d9ad4 -- Cargo.toml Cargo.lock emissary-core/src/sam/protocol/streaming/config.rs emissary-core/src/sam/protocol/streaming/mod.rs emissary-core/src/sam/session.rs emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml` | empty |
| `git diff 6d631d4423c7faa761b47a84e07436bbaf5d9ad4..HEAD --name-status` | only exact planning/bookkeeping paths (`092` plan, `093` plan, `092`-closure, M091 plan status annotation, README/registry/roadmap updates, M062 test allowlist entry); no production/dependency paths |

`cargo fmt --all -- --check` was not run on this milestone; the repository's existing rustfmt/nightly configuration drift is not authorization for formatter-only churn under M092. The M090 closure record notes the same pre-existing formatting drift on this repository.

### M090 regression evidence

- `cargo test -p emissary-cli --no-default-features --features i2pcontrol http_server::tests` → pass; loopback target typing, HTTP-bidir typed seam, and M090 normalization tests green.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol irc_server::tests` → pass; literal-loopback IRC targets and M090 half-close drain tests green.

## 8. M088 lower-layer residual disposition

After M092 the post-M091 accepted-server architecture is exactly:

```text
remote signed streaming SYN
  -> Emissary lower-layer streaming parse/signature/replay work
  -> pending/active stream-manager and routing/SAM work
  -> Yosemite Session<style::Stream>::accept()
  -> TrustedPeerIdentity
  -> ServerAdmissionState
  -> bounded application handler / local target
```

This is the M088 boundary. Signed-SYN/streaming work can occur before `ServerAdmissionState` runs. The M091 pre-allocation stream-concurrency check has been intentionally removed because its dependency transport was not authorized under the registered handoff. This is a known availability/timing residual, not a direct clearnet identity leak, and is the current accepted disposition under M088.

## 9. Security invariant preservation

| Invariant | Result |
|---|---|
| Proposal 170 API spelling/types/actions/tunnel set | preserved unchanged |
| All twelve real tunnel backends | preserved unchanged |
| M090 literal-loopback target normalization | preserved unchanged |
| M090 IRC half-close drain | preserved unchanged |
| Authenticated Yosemite/SAM trusted peer identity | preserved unchanged |
| Bounded post-accept `ServerAdmissionState` concurrency/rate/cardinality | preserved unchanged |
| HTTP framing/identity/fingerprint/POST protections | preserved unchanged |
| IRC bounded registration/connect/inactivity | preserved unchanged |
| Streamr bounded local-only fanout model | preserved unchanged |
| Persistent server key confinement/redaction | preserved unchanged |
| Startup/control-plane ownership separation | preserved unchanged |
| No private Destination/key material in diagnostics | preserved unchanged |
| No upstream interaction | confirmed |

## 10. Explicit non-goals confirmed

M092 did not:

- redesign the lower-layer admission mechanism;
- retain a partial Yosemite vendor copy;
- replace vendoring with a git/path dependency or local patch;
- add raw SAM command construction or a hidden registry;
- port Java I2P `ConnThrottler` behavior;
- modify tunnel lengths, crypto, router selection, NetDb, transport, or routing algorithms;
- change Streamr fairness or authentication;
- change `httpbidirserver` identity sharing;
- add new Proposal 170 fields/actions/types;
- add hosted CI, fuzz, soak, release, or public-network load machinery;
- prepare or request upstream review/merge/submission.

## 11. Unresolved findings

No high- or medium-severity production security/anonymity defects were found in this corrective pass.

Accepted residual limitations carried forward (already documented in M088, M090, M091 closures, and the subsystem roadmap):

- M088 lower-layer pre-accept / signed-SYN timing residual: authenticated signed-SYN/streaming work can occur before `ServerAdmissionState` runs. Severity: medium (availability/timing); not a direct clearnet identity leak. M093 must re-record this as the current accepted residual after the M092 rollback.
- M087 progress-based generic-server inactivity behavior and M090 half-close/drain behavior: preserved unchanged.
- Streamr finite subscriber set Sybil-monopolization availability limitation: not claimed as fixed; not in M092 scope; tracked in the subsystem roadmap.

## 12. Internal-only attestation

All implementation, planning, closure, registry, and roadmap writes were confined to the internal `eggstack/emissary` repository. No upstream issue, PR, review, submission, merge request, maintainer contact, contribution artifact, vendored-crate submission to crates.io, or external repository write was opened, drafted, requested, or pushed. External I2P, I2P+, Yosemite, and reference source repositories and specifications remain read-only evidence.

## 13. Dependency decision for M093

M092 closure is accepted. M093 (`plans/implementation/i2pcontrol-proposal-170/093-post-m092-tunnel-security-reclosure.md`) is promoted from `blocked` to `ready` and becomes the only dependency-ready tunnel-security implementation handoff. M093 is a no-production-change independent security reclosure of all twelve Proposal 170 tunnel backends at the corrected head. Any new high/medium production defect found by M093 must open a new numbered corrective and keep M093 blocked rather than modifying production code under M093.

The tunnel-security line is not closed by M092. It becomes current-head closed only after an accepted M093 closure.
