# M059 Closure Record — Original CLI and Runtime Adapter Containment

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/059-cli-runtime-containment.md`

Accepted predecessor and budget:

- `plans/closure/i2pcontrol-proposal-170/058-closure.md`
- `plans/implementation/i2pcontrol-proposal-170/058-containment-ledger.toml`

Repository baseline reviewed: `adb2f52543764b267b2bcb282d093111001ae4b`.

Implementation head: `ed17fe7` (`contain i2pcontrol policy in cli runtime adapters`).

Planning head when closed: the closing planning commit containing this record.

Upstream comparison baseline: `9b43484a21d5a1291c4881cdae62a36c527f8c0f`
(`eepnet/emissary` fork merge base, used read-only).

## 1. Executive disposition

M059 is closed. The original CLI/runtime containment budget was implemented
without changing `emissary-core/**`, the Proposal 170 wire contract, persisted
control-state schema, or supported runtime lifecycle behavior.

Administrative AddressBook DTOs, validation, persistence, migration/repair,
subscription commit semantics, and the dedicated administrative handle now
reside in `emissary-cli/src/i2pcontrol/address_book_runtime.rs`. The original
AddressBook retains ordinary downloading, parsing, legacy-file persistence,
and one narrow neutral hook interface. Runtime lookup remains authoritative
when the control owner is attached, so deleting an administrative entry cannot
resurrect a stale legacy destination file.

The accepted RouterInfo disposition remains 37 available / 1
protocol-permitted neutral / 5 unavailable. Unsupported tunnel backends remain
resource-free and explicitly unsupported. M060 is unblocked and is now the
sole dependency-ready containment handoff. M061 remains planned behind M060;
M051 remains independently blocked by absent substantive news/ban owners.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| M058 original-CLI budget respected | Baseline-to-implementation path audit | pass | Changed original paths are in the frozen M059 budget; policy moves use the authorized `i2pcontrol/**` area |
| No core path changed | Baseline diff and `m059_containment` guard | pass | No `emissary-core/**` path in implementation or closure changes |
| Administrative policy is under I2PControl | Owner review and source guard | pass | DTOs, bounds, state, persistence, migration, and subscription semantics moved out of original AddressBook |
| Original AddressBook retains only ordinary runtime plus a neutral seam | `m059_containment` | pass | No JSON-RPC terms, control-state schema, or Proposal 170 policy in production code |
| Proxy/tunnel modules contain no control-plane policy | `m059_containment` and upstream diff review | pass | HTTP candidate changes reverted; SOCKS retains only a read-only listener-address accessor |
| Supported behavior is preserved | AddressBook, composition, ClientServicesInfo, and tunnel lifecycle suites | pass | Focused regression suites pass |
| Unsupported backends remain safe | Existing production composition/backend tests | pass | No unsupported data-plane implementation or allocation was added |
| RouterInfo matrix unchanged | Accepted M058/M056 invariant review | pass | 37/1/5 remains authoritative |
| No dependency or persistence migration | Manifest/lockfile and source review | pass | No manifest, lockfile, schema, config-key, or wire change |
| Internal-only boundary preserved | §8 attestation | pass | Internal `origin` push only; no upstream write or review activity |

## 3. Before/after ownership

| Area | Before M059 | After M059 | Disposition |
|---|---|---|---|
| Original AddressBook | Ordinary downloader/resolver mixed with runtime owner construction, administrative validation, state migration, subscription control, and control-state persistence | Ordinary downloader/resolver plus `AddressBookRuntimeHook`; I2PControl supplies the attached owner | Policy extracted; one canonical owner retained |
| AddressBook control owner | Interleaved with original manager/handle types | `i2pcontrol::address_book_runtime::{RuntimeAddressBookOwner, RuntimeAddressBookHandle}` | Administrative owner physically contained |
| Main composition | Constructed the old manager-owned control owner and retrieved its handle | Calls `new_controlled_manager` and passes the handle to composition | Composition only |
| HTTP error/request helpers | Formatting-only fork delta | Pinned upstream behavior restored exactly | Candidate-revert paths closed |
| SOCKS proxy | Formatting-only delta plus required bound-listener accessor | Formatting restored; neutral `local_addr` accessor retained | Required owner seam |
| HTTP proxy owner / tunnel client/server | Existing neutral lifecycle seams | Unchanged | No M059 policy was present |
| Config, library exports, logger | Existing feature/composition scaffolding | Unchanged | No direct removal was proven safe or necessary |

## 4. Exact changed-path proof

Changed production paths in implementation commit `ed17fe7`:

```text
emissary-cli/src/address_book.rs
emissary-cli/src/i2pcontrol/address_book_runtime.rs
emissary-cli/src/i2pcontrol/production.rs
emissary-cli/src/i2pcontrol/server.rs
emissary-cli/src/main.rs
emissary-cli/src/proxy/http/error.rs
emissary-cli/src/proxy/http/request.rs
emissary-cli/src/proxy/socks.rs
```

Focused test paths changed:

```text
emissary-cli/tests/adversarial.rs
emissary-cli/tests/m059_containment.rs
emissary-cli/tests/production_adapter.rs
emissary-cli/tests/production_composition.rs
```

No `config.rs`, `lib.rs`, `logger.rs`, `proxy/http/mod.rs`,
`tunnel/client.rs`, or `tunnel/server.rs` change was necessary. No manifest or
lockfile change was necessary. No `emissary-core/**` path changed.

Candidate-revert results:

- `proxy/http/error.rs` and `proxy/http/request.rs` match the pinned upstream
  formatting and match-arm shape again;
- `proxy/socks.rs` has the formatting-only hunk restored, while retaining the
  ten-line read-only `local_addr` accessor required by existing passive
  ClientServicesInfo observation.

## 5. Behavioral and security review

- The neutral hook carries bounded commands and owner-local snapshots only; it
  does not expose sockets, keys, sessions, payloads, or mutable router authority.
- Runtime control state has one persistence owner. The ordinary AddressBook no
  longer parses or writes `control-state.json` or keeps a duplicate admin map.
- Validate-then-persist-then-activate mutation ordering is unchanged.
- Feature-off construction uses the ordinary AddressBook path and does not
  construct I2PControl state.
- Subscription refresh retains its bounded command/worker and cancellation
  behavior; no lock spans network or filesystem `.await` points.
- Proxy observation remains passive and cannot control service lifecycle.
- No new task, event bus, polling loop, persistent store, dependency, or
  unsupported tunnel backend was introduced.

## 6. Verification executed

Passed:

```text
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features address_book
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib address_book::tests
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m059_containment --test m037_containment --test production_composition --test client_services_integration --test m033_tunnel_lifecycle
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

The feature-on integration invocation completed 34 tests across the five
requested suites; the AddressBook feature-on library filter completed 90
tests. The first AddressBook run caught a lookup-precedence regression; the
neutral hook was corrected to remain authoritative when attached and the full
focused suite was rerun successfully.

The installed stable/nightly rustfmt versions report pre-existing
repository-wide formatting drift, including untouched core, utility, and
I2PControl files. `cargo fmt --all -- --check` and
`cargo +nightly fmt --all -- --check` therefore do not provide a clean result
without rewriting unrelated paths. The M059 diff has no whitespace errors
(`git diff --check` passes), and no broad formatting rewrite was made because
that would violate the frozen M059 boundary and no-core-change invariant.

## 7. Dependency and future-plan disposition

No dependency, export contract, configuration key, or persistence format was
removed or changed. The composition entry point is now
`i2pcontrol::address_book_runtime::new_controlled_manager`; the old manager
control-owner constructor/accessor no longer exists in ordinary AddressBook.

M059 is closed. M060 is ready against implementation head `ed17fe7` and the
exact 32-path `budgets.m060_core` list in the accepted M058 ledger. M061 remains
planned and hard-blocked on M060. No other future plan became ready. M051
remains blocked by absent substantive news and banned-peer owners.

## 8. Internal-only attestation

Upstream source and commit identity were accessed read-only for comparison. No
upstream repository or maintainer channel was mutated; no issue, pull request,
review, merge, adoption request, submission, contribution artifact, branch,
tag, or release was created or prepared. The requested push targets the
internal `eggstack/emissary` `origin` only. All writes remain within this
repository.

**Disposition: M059 closed; original CLI/runtime containment accepted; M060 ready.**
