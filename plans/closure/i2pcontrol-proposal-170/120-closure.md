# M120 Closure — Server Start Preallocation Validation and Secret Transactionality Corrective

Status: **closed**

Plan: `plans/implementation/i2pcontrol-proposal-170/120-server-start-preallocation-validation-and-secret-transactionality-corrective.md`

Implementation commit: `627309c`

Closure date: 2026-09-03

## Disposition

M120 is closed. Every deterministic server start validation now fails before
private destination allocation, import, or persistence, and any remaining
server-secret mutation is transactional across runtime start failure. The
control-plane start order is now load → common validation → backend pure
preflight → staged identity preparation → backend start → commit of the secret
and the public destination. No Proposal 170 matrix cell is promoted or
demoted and M095 remains `312 apply / 70 blocked_primitive / 458
not_applicable`.

Prior M110/M116/M113 history is untouched. One latent defect found by the new
restart regression (the generic server capability set rejected the
control-plane-persisted `HostingDestination` display field, so no committed
generic server could ever restart) is corrected inside the same seam; the
other four server families already accepted that field and no matrix cell
covers it.

## Preflight coverage freeze (WP1)

`TunnelBackend::validate_start` is the single pure seam in
`emissary-cli/src/i2pcontrol/backends/mod.rs`. Its contract is explicit: no
listener/session/task allocation, no network I/O, no private destination
generation/import/store mutation, and no runtime-map reservation. Every
server `start` calls its own `validate_start` first, so preflight and actual
start reuse the same helpers and cannot drift. Unsupported backends fail
preflight with `NotImplemented`; client backends keep the default accept so
client staging order is unchanged.

| Family | Deterministic gates in `validate_start` | Dynamic (rollback-covered) |
|---|---|---|
| `server` | ownership, common options, raw allowlist, I2CP allowlist (`leaseSetEncType` only), typed `SERVER_OPTIONS`, port/loopback/admission/access/LeaseSet shape via `runtime_config`, session-wire ranges (`TunnelLength`/`TunnelQuantity`/`EncType`/`SigType`/variance/custom) via `build_session_options` with a dummy transient destination | secret-store lookup, SAM session, supervisor reserve/capacity, public-destination persistence |
| `httpserver` | common options, `config_without_destination` (ownership, raw, typed, target/loopback, website/spoofed, access incl. bounded local filter-file read, admission, post limits), session-wire via dummy destination | identity/store lookup, SAM session, supervisor, persistence |
| `httpbidirserver` | common options, `config_without_destination` (above plus listen/bind/proxy-auth/client policy), server and client session-wire via dummy destinations | same as above |
| `ircserver` | common options, `config_without_destination` (ownership, typed, raw, admission, access, loopback target), session-wire via dummy destination | same as above |
| `streamrserver` | shared `config` with a dummy destination (ownership, typed, common, raw Streamr allowlist, listen port, loopback bind) | identity/store lookup, UDP/SAM runtime, persistence |

Filter-file reads inside preflight are bounded local filesystem validation
only — never network I/O, secret work, or runtime reservation — and their
failures now also precede allocation. `config_without_destination` for the
HTTP and IRC servers was narrowed from needlessly `async` to sync so the
sync preflight can reuse it without drift; bodies had no awaits.

## Requirement-to-evidence matrix

| Requirement | Evidence and outcome |
|---|---|
| Deterministic failures precede allocation (§3.2, criteria 1) | `start_locked` runs common validation then `validate_start` before any staging; 12 production regressions assert zero identity keys, zero staged secrets, and zero SAM connections for common/raw/I2CP failures across all five families, plus import-order evidence (missing import file + invalid option reports the option). |
| Exact secret/durable rollback (criterion 2) | `ServerDestinationStore` gains owner-local staging (`stage` in-memory only, `commit` persists, `discard` drops, `get` shadows staged, `load` clears, `put`/`remove` supersede staging). Fresh starts never persist the identity-bearing definition before commit; replacements shadow durability until commit. Regressions cover replacement restore, fresh import/generated rollback with no durable file, and public-destination persistence failure (secret uncommitted/restored, runtime stopped). |
| Single-identity commit (criterion 3) | Generated success regression commits exactly one identity/secret/public destination and preserves it across stop/restart/reload with the same private material and a bound public destination. |
| No secret exposure (criterion 4) | Failure strings assert no private material, import values, or filenames; store/backend `Debug`/`Display` redaction retained and asserted; previous-secret bytes for commit-phase restore never enter logs or errors. |
| Concurrency/cancellation safety (criterion 5) | Per-name lifecycle lock still spans preflight through commit/rollback; no store/runtime lock is held across network I/O. Concurrent same-name starts commit exactly one identity (one file entry). A `ServerStartGuard` synchronously discards staging on cancellation/drop; an abort-mid-start regression asserts zero pending, zero identity, and zero durable file. |
| I2PControl confinement (criterion 6) | Production changes are `emissary-cli/src/i2pcontrol/**` plus the exact M062 seam; no core/util/CLI/Yosemite/frontend/workflow/dependency change. M061 manifest unchanged (policy root); M062 gains `is_authorized_m120_path`. |
| Verification (criterion 7) | See tables below; only baseline rustfmt drift is dispositioned. |
| M121 readiness (criterion 8) | M121 is unblocked (ready); M122 remains blocked on M121 + Y004; see readiness audit. |

## Changed paths

Implementation commit `627309c`:

- `emissary-cli/src/i2pcontrol/backends/mod.rs` — pure `validate_start` seam with default accept;
- `emissary-cli/src/i2pcontrol/backends/server.rs` — generic preflight + `start` reuse + preflight test;
- `emissary-cli/src/i2pcontrol/backends/http_server.rs` — sync narrowing, preflight + reuse + test;
- `emissary-cli/src/i2pcontrol/backends/http_bidir.rs` — preflight + reuse + test;
- `emissary-cli/src/i2pcontrol/backends/irc_server.rs` — sync narrowing, preflight + reuse + test;
- `emissary-cli/src/i2pcontrol/backends/streamr.rs` — server preflight in the trait impl (not the inherent block) + reuse + test;
- `emissary-cli/src/i2pcontrol/backends/unsupported.rs` — preflight fails `NotImplemented`;
- `emissary-cli/src/i2pcontrol/backends/options.rs` — generic server accepts persisted `HostingDestination` (restart parity, no matrix cell);
- `emissary-cli/src/i2pcontrol/production.rs` — corrected start order, `PreparedServerStart`/`ServerStartGuard` transaction, commit/rollback, 12 regressions;
- `emissary-cli/src/i2pcontrol/server_secret_store.rs` — staging/commit/discard/shadowing plus 3 transaction tests;
- `emissary-cli/tests/m062_dependency_containment.rs` — `is_authorized_m120_path` wired into both budget asserts.

Closure commit changes only planning evidence:

- this record;
- `plans/implementation/i2pcontrol-proposal-170/120-server-start-preallocation-validation-and-secret-transactionality-corrective.md` status to closed;
- `plans/implementation/i2pcontrol-proposal-170/121-m111-m112-semantic-truthfulness-corrective.md` promotion to ready;
- `plans/registry.md`;
- `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md`.

No `emissary-core/**`, `emissary-util/**`, `emissary-cli/src/main.rs`, Cargo/dependency, Yosemite, frontend, workflow, or release production change.

## Focused and broad verification

All commands were run from the repository root:

| Command | Outcome |
|---|---|
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | PASS |
| `cargo check -p emissary-cli --no-default-features` | PASS |
| `cargo check` | PASS |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib` | PASS — 698 tests (678 baseline + 20 new M120) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | PASS — all 26 suites, zero failures |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit` | PASS — 33 tests (7 + 23 + 2 + 1) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture` | PASS — 1 test |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | PASS — no issues |
| `git diff --check` | PASS |
| `cargo fmt --all -- --check` | FAIL due the repository's pre-existing stable/nightly rustfmt configuration/toolchain mismatch; it reports broad formatting churn in untouched files, and no formatter churn was retained |
| M095 matrix counts | Unchanged: `312 apply / 70 blocked_primitive / 458 not_applicable` |

Focused M120 regressions (all in existing test modules, no new test files):

- production (12): common/raw/I2CP preallocation failure for all five families with SAM call-count evidence; import-order evidence; never-persisted identity keys incl. reload; replacement restore; fresh import/generated rollback with no durable file; persistence-failure rollback with stopped runtime; success commit across stop/restart/reload; concurrent single commit; cancellation guard; startup-managed path unchanged;
- backends (5): per-family preflight/start agreement without store or session;
- secret store (3): stage shadow/commit/discard semantics, no durable write until commit, sync guard discard, direct-write supersession, load clearing.

## Failure, cancellation, contention review

- Validation failures return `Ok("error - ...")` naming the option without allocating, importing, generating, staging, or contacting SAM.
- Dynamic failures (SAM unreachable/session close, missing public destination, persistence failure) stop the runtime and discard or restore staged state; fresh failures leave no durable file and the original definition; replacement failures keep the exact previous secret and definition.
- Crash between staging and commit loses only memory: fresh orphans were never durable (existing prune remains a backstop) and replacements keep the old durable secret authoritative.
- The per-name lifecycle lock covers preflight through commit/rollback; concurrent same-name starts serialize and commit once.
- Cancellation after staging drops the guard-held candidate via `discard_sync`; state locks are held only for short in-memory updates, never across network or file I/O.

## Compatibility, migration, security review

- No storage migration: staging is memory-only; durable formats are unchanged. Old committed definitions reload unchanged; the generic `HostingDestination` acceptance additionally repairs their restart.
- No public JSON-RPC, SAM, configuration, dependency, or wire change.
- Secrets never enter responses, `RawConfig`, logs, `Debug`/`Display`, `RouterInfo`, or public-destination fields; failure strings are asserted secret-free.
- Literal-loopback, proxy, HTTP/IRC/Streamr, admission, and lease boundaries are untouched; M061/M062 guards pass.
- Startup-managed definitions still delegate to the external lifecycle owner and never enter the secret transaction.

## Future-plan readiness audit

- **M121 — ready.** M120 closure satisfies its hard gate (`blocked on M120 closure`). Its I2PControl-only `SigType`/`Close`/`CloseTime`/`NewDest` truthfulness work may now be handed off.
- **M122 — remains blocked.** Still gated on M121 and Yosemite Y004 closure.
- **Y004 — separately ready in Yosemite.** No Emissary action; Y003 remains unconsumed.
- No future plan other than M121 became unblocked. M095 counts remain a capability baseline and no infrastructure correction was treated as Proposal support.

## Unresolved findings

1. **Low — tooling:** the repository's committed rustfmt configuration is not accepted consistently by the installed stable/nightly formatters. This is pre-existing toolchain drift, not an M120 source defect.
2. **Low — error-string wart (pre-existing):** backend `BackendError` display strings already start with `error -`, so control-plane wrapping yields `error - error - ...` for some failures. Left unchanged to minimize scope; all assertions match on `starts_with("error")` plus the option name.
3. **None — high/medium.** No open M120 security, containment, correctness, or lifecycle finding remains.

## Internal-only attestation

External specifications and reference-router sources were inspected read-only. No upstream repository, issue, pull request, review, maintainer channel, release artifact, or external branch/tag was mutated or requested. All implementation and planning writes are internal to `eggstack/emissary`.
