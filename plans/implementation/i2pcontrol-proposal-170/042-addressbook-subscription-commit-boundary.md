# M042 — AddressBook Subscription Commit-Boundary Correction

Status: closed

Hard dependency:

- M041 closed

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Corrective authority:

- `plans/closure/i2pcontrol-proposal-170/039-closure-invalidation.md`

Applicable governance and retained evidence:

- `plans/003-planning-process.md`
- M028/M030 AddressBook owner and feature-isolation records
- M034 implementation and closure records

Repository defect baseline:

- `563e093ba1e65b4edc31104e3045c8b5a665e8ed`

## 1. Bounded objective

Give `SetSubscriptions` one truthful mutation boundary: a request must never
report failure after the replacement subscription set has been durably committed
and made active.

The supported operation is replacement of the live manager's subscription set.
Immediate successful retrieval of every subscription is follow-up work, not part
of the mutation transaction. Refresh remains bounded and owned by the existing
AddressBook manager.

M042 preserves explicit rejection of non-empty `SetConfig`, fixed path ownership,
one enabled-mode AddressBook authority, and disabled/default behavior.

## 2. Demonstrated defect and prior evidence gap

The current manager handles a command by:

1. durably committing the subscription replacement;
2. updating the active subscription set;
3. attempting to enqueue refresh work;
4. returning an error if the refresh worker is unavailable.

If step 3 fails, the caller receives failure even though steps 1 and 2 already
occurred. Retrying cannot distinguish no mutation from an already-completed
replacement.

Existing tests prove persistence and live manager control but do not terminate
or close the refresh worker after the durable commit and assert the resulting
wire status.

## 3. Required operation contract

`SetSubscriptions` success means:

- the request was validated;
- the live manager accepted the replacement;
- the replacement was durably published by the authoritative owner;
- the manager's active source set reflects the replacement.

It does not mean that every remote source was fetched or merged before the
response.

After durable commit:

- refresh scheduling is best-effort bounded follow-up work;
- a scheduling failure may be logged with sanitized diagnostics;
- the request must still return success because the mutation is complete;
- the committed set remains available for later refresh or restart.

Before durable commit:

- unavailable command ownership, invalid input, publication failure, or
  cancellation must return error and leave prior durable/active state unchanged.

## 4. Required invariants

1. No error is returned after durable mutation commit.
2. No success is returned before durable commit and active-set publication.
3. Validation occurs before mutation.
4. Command-channel failure before manager receipt performs no mutation.
5. Publication failure preserves prior durable and active state.
6. Refresh failure cannot roll back the accepted subscription set.
7. Refresh work remains bounded and coalesces to the newest complete set.
8. No second AddressBook authority, scheduler, event bus, or general job system.
9. Non-empty `SetConfig` remains deterministic unsupported/error.
10. Request-selected file paths remain rejected or unavailable.
11. M028 disabled/default feature isolation remains exact.
12. M030 full-destination owner coherence remains exact.
13. No core change.
14. No upstream interaction.

## 5. Scope and production file budget

### Primary production files

- `emissary-cli/src/address_book.rs`
- `emissary-cli/src/i2pcontrol/address_book_runtime.rs` only if the command/result
  type needs a narrow clarification
- `emissary-cli/src/i2pcontrol/address_book.rs` only if wire-status translation
  requires a directly related correction

### Authorized tests and records

- focused AddressBook manager/runtime tests;
- existing production-adapter and live-runtime tests where directly affected;
- M042 implementation disposition and closure record;
- directly affected AddressBook documentation.

### Prohibited changes

- `emissary-core/**`;
- legacy/default AddressBook ownership redesign;
- new download scheduler, queue framework, event bus, or background service;
- arbitrary file/config fields;
- new AddressBook methods or aliases;
- tunnel/auth/RouterInfo/ClientServicesInfo changes;
- CI/release expansion.

## 6. Target command sequencing

The manager command path must use a sequencing equivalent to:

1. receive one bounded replacement command;
2. validate the complete set;
3. clone the prior active state if needed for failure preservation;
4. durably mutate the authoritative owner;
5. publish the same set into the manager's active state;
6. resolve the command response as success;
7. enqueue/coalesce refresh work best-effort;
8. log a bounded sanitized warning if refresh scheduling is unavailable.

An alternative sequence may verify worker availability before commit, but it
must still guarantee that any post-commit worker failure cannot turn a completed
mutation into an error response. The implementation must document the selected
linearization point.

The operation must not wait for network download completion.

## 7. Ordered work packages

### WP1 — Add the failing post-commit worker regression

Create a deterministic test seam that allows the refresh worker or refresh
sender to become unavailable after command receipt and durable commit.

Assert that:

- the request returns success;
- durable subscriptions contain the replacement;
- active subscriptions contain the replacement;
- no stale prior set is reported;
- a bounded diagnostic is emitted or otherwise observable only internally;
- restart loads the committed replacement.

The test must fail on baseline `563e093` because the current path returns an
error after commit.

### WP2 — Freeze pre-commit failure behavior

Add tests for:

- command channel unavailable before manager receipt: error, no mutation;
- validation failure: error, no mutation;
- publication failure/cancellation: error, prior active and durable state
  retained;
- manager not started: documented unavailable error, no mutation.

### WP3 — Correct the linearization point

Refactor only the local command response/scheduling sequence. Avoid introducing
a general transaction abstraction.

The manager must respond according to the durable mutation result. Refresh
scheduling must not alter that result after commit.

### WP4 — Revalidate coalescing and bounds

Prove that:

- only one bounded command slot and one bounded refresh slot remain;
- multiple replacements coalesce refresh to the newest complete set;
- each caller receives the result of its own durable mutation;
- refresh backlog cannot grow without bound;
- no lock is held across download/retry sleeps;
- downloaded entries still merge through the one runtime owner.

### WP5 — Clarify documentation

Document that `SetSubscriptions` applies the active source set durably and
schedules refresh; it does not synchronously guarantee remote retrieval.

Document that positive refresh may remain unavailable when the HTTP downloader
is not composed, but that unavailability must be determined before mutation or
reported only as follow-up diagnostic after a completed mutation.

## 8. Failure, cancellation, restart, and contention semantics

- Cancellation before the owner publishes leaves prior state unchanged.
- Cancellation after the owner publishes cannot make the completed operation
  appear failed; the manager may complete the response or treat the operation
  as committed.
- Refresh worker failure after commit is non-transactional follow-up failure.
- Restart reads the newest committed subscription set.
- Concurrent commands serialize through the existing bounded manager seam.
- Refresh work coalesces to the newest committed set.
- Network failures do not roll back subscriptions.
- No global lock is held across HTTP I/O or retry backoff.

## 9. Compatibility and migration

- No JSON-RPC method, selector, field, or response-shape change.
- No AddressBook state schema migration unless strictly required; preference is
  no schema change.
- Existing committed subscription sets remain readable.
- Non-empty `SetConfig` remains unsupported.
- Disabled/default builds remain unaffected.

## 10. Required tests

At minimum:

1. refresh worker closes after durable commit: success and committed state;
2. command receiver unavailable before receipt: error and no mutation;
3. manager not started: error and no mutation;
4. validation failure: error and no mutation;
5. publication failure: error and prior state retained;
6. concurrent replacements preserve per-command mutation results;
7. refresh coalesces to newest set without unbounded queue growth;
8. restart restores committed replacement;
9. non-empty `SetConfig` remains explicit unsupported;
10. no-feature and runtime-disabled paths do not initialize or consult control
    state.

## 11. Verification commands

At minimum:

```bash
cargo test -p emissary-cli --no-default-features address_book
cargo test -p emissary-cli --no-default-features --features i2pcontrol address_book
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test golden_fixtures
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

The live test may retain a documented no-downloader configuration, but focused
manager tests must prove the post-commit worker-failure case directly.

## 12. Documentation and static guards

Update:

- `docs/i2pcontrol/address-book.md`;
- `docs/i2pcontrol/proposal-170-support.md` if its setter wording implies
  synchronous download success;
- M042 disposition and closure.

Do not add a broad static scanner. Existing feature-boundary guards should be
re-run unchanged.

## 13. Acceptance criteria

M042 may close only when:

- `SetSubscriptions` has one documented durable linearization point;
- no post-commit path returns mutation failure;
- pre-commit failures leave prior state unchanged;
- refresh remains bounded follow-up work;
- restart preserves the committed replacement;
- no general scheduler, second owner, path API, or schema redesign was added;
- disabled/default and owner-coherence evidence remains valid;
- no unresolved high- or medium-severity finding remains in this slice.

## 14. Stop conditions

Stop rather than:

- wait synchronously for all downloads;
- create a generic job scheduler or event bus;
- add arbitrary AddressBook paths/configuration;
- create a second authority or bidirectional store synchronizer;
- modify core;
- expand CI/release infrastructure;
- interact with upstream.

## 15. Closure evidence required

The M042 disposition and closure must include:

- implementation/test commit SHA;
- explicit mutation linearization point;
- failing-before/passing-after post-commit worker evidence;
- pre-commit failure preservation evidence;
- coalescing/capacity evidence;
- restart evidence;
- verification command outcomes;
- changed-path classification;
- unresolved findings with severity;
- internal-only/no-upstream attestation.
