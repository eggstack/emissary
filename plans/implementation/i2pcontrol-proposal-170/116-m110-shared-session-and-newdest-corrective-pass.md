# M116 — M110 Shared-Session, Streamr Isolation, and NewDest Corrective Pass

Status: **ready**

Class: corrective concurrency / security / lifecycle semantics / truthfulness / containment

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Corrective authority and predecessor evidence:

- M110 plan: `plans/implementation/i2pcontrol-proposal-170/110-shared-client-session-and-destination-key-ownership-completion.md`
- M110 closure: `plans/closure/i2pcontrol-proposal-170/110-closure.md`
- M110 completion ledger: `plans/implementation/i2pcontrol-proposal-170/110-completion-ledger.toml`
- M115 closure: `plans/closure/i2pcontrol-proposal-170/115-closure.md`
- M093 security closure
- M061/M062/M063 containment authority
- M095 full-support matrix and M105 residual-option audit

Repository baseline:

- `09247ccf8367a7b3a7050e0584614c4e59cafe8e` — post-M110 closure/containment head.

Pinned external authority:

- I2P Proposal 170, `I2PControl Expansion`, status `Open`, revision `2026-05-20`;
- Java I2PTunnel behavior used only as read-only semantic evidence where Proposal 170 does not define the lower-level lifecycle trigger precisely;
- Tokio `Notify` behavior used only as read-only concurrency API evidence;
- accepted Yosemite `0.7.0` public API/source used only as read-only dependency evidence.

All external sources are read-only. This plan authorizes writes only to `eggstack/emissary`. It does not authorize upstream issues, pull requests, review requests, maintainer contact, submission/merge/adoption work, release activity, or writes to any external repository.

## 1. Objective

Correct the bounded M110 defects discovered after closure without reopening the broader Proposal 170 program, changing Yosemite, or moving policy into the audited router/core crates.

M110 added real I2PControl-local shared-session and client-identity ownership, but post-closure review found correctness/security defects in that implementation and a semantic problem in the `NewDest` matrix promotion:

1. shared-session waiters can miss `Notify::notify_waiters()` and sleep indefinitely;
2. cancellation of the task currently creating a shared session can leave the compatibility entry permanently stuck in `creating=true`;
3. the shared-session compatibility key reduces persistent private identity to a 64-bit `DefaultHasher` fingerprint, so the implementation does not provide collision-safe proof that incompatible identities never share;
4. shared Streamr clients receive a broadcast of every inbound datagram and currently forward payloads without proving that the Yosemite-authenticated peer is the configured producer, allowing cross-member/cross-producer delivery;
5. M110 currently treats `NewDest` as "generate a new identity whenever a start is staged", while reference I2PTunnel `newDestOnResume` semantics are coupled to close-on-idle/resume and incompatible with persistent-client-key behavior; M112 still owns the required `Close*` lifecycle family;
6. internal client-secret store structs derive raw `Debug` while containing private destination material, leaving an unnecessary accidental disclosure surface even though the public wrapper is redacted.

M116 must correct these defects and re-establish a truthful M110 disposition before any successor milestone is considered.

The current M095 counts `255 apply / 127 blocked_primitive / 458 not_applicable` are **provisional during M116**. Matrix counts are evidence, not a target. Closure must calculate the exact post-corrective counts from runtime/semantic evidence.

## 2. Why M110 closure remains historical

Do not rewrite `plans/closure/i2pcontrol-proposal-170/110-closure.md` as if the post-closure findings had never existed.

The repository planning process treats later-discovered material defects as a separately numbered corrective. M110 therefore remains historical closure evidence for implementation commit `56c8459...`; M116 records the corrective delta and becomes the current authority for whether M110's 31 promoted cells remain supported.

At M116 closure:

- M110's closure remains unchanged except for references from newer control surfaces;
- M116 closure records which M110 cells remain `apply`, which return to `blocked_primitive`, and why;
- M095 is the current authoritative matrix;
- M110 completion ledger and M105 historical-audit reconciliation may receive explicit post-M116 fields, but historical input evidence must not be falsified or silently rewritten.

## 3. Findings requiring correction

### F1 — lost wakeup in shared-session acquisition

`SharedClientSessionRegistry::{acquire_stream,acquire_datagram}` currently:

1. locks the registry;
2. sees an existing entry whose creation is in progress;
3. clones `Arc<Notify>`;
4. drops the registry lock;
5. only then constructs/awaits `notify.notified()`.

The creator publishes success/failure with `notify_waiters()`.

Tokio's `notify_waiters()` does not store a permit for a future waiter. Therefore creation can finish between steps 4 and 5; the waiter then waits for a notification that already occurred and may sleep indefinitely.

Disposition: **medium correctness/reliability defect**.

M116 must make waiter registration and state observation linearizable. Acceptable patterns include a state-generation/watch/oneshot design or constructing/enabling the wait future before a notification can be lost. Do not solve this by holding the registry mutex across `Session::new(...)` or other network I/O.

### F2 — creator cancellation can poison one compatibility key

Session construction occurs outside the registry lock after setting `creating=true`. If the future performing `Session::<...>::new(...)` is dropped because listener/session setup is cancelled, the cleanup path that clears/removes the creator reservation is not guaranteed to run.

A later acquisition can then observe:

- `session=None`;
- `creating=true`;
- no live creator capable of publishing completion.

Combined with F1, this can leave compatible starts waiting indefinitely.

Disposition: **medium correctness/recovery defect**.

M116 requires cancellation-safe creator reservation. The reservation must be cleared or transferred if the creator future is dropped, cancelled, panics, times out, or returns error. Waiters must be woken/retry against current state. No abandoned `creating=true` entry may survive without an active creator.

### F3 — compatibility identity is not collision-safe

`compatibility_key()` currently replaces persistent destination material with a 64-bit `DefaultHasher` result before building the registry key.

M110's invariant is stronger than "collision is unlikely": definitions with different persistent security identities must never share by accident. A truncated/non-cryptographic fingerprint cannot provide exact identity separation.

Disposition: **medium security/correctness defect**.

M116 must use collision-safe compatibility equality for persistent identity. Preferred designs:

- a private structured compatibility key whose equality/order compares the exact private identity while its `Debug`/`Display` is redacted; or
- an already-accepted cryptographic digest primitive with adequate collision resistance, if available without adding a dependency or exposing secret material.

Do not add a dependency solely for this correction. Do not expose the identity or derived secret-bearing key in logs, RPC, ordinary `Debug`, raw config, RouterInfo, or ClientServicesInfo.

`SessionOptions` compatibility must continue to include every session/security setting actually translated into Yosemite, while deliberately excluding only fields such as per-tunnel nickname that are proven non-contractual for session sharing.

### F4 — shared Streamr application isolation is not enforced

`SharedDatagramSession` broadcasts each inbound `{payload, peer}` to all subscribers. `run_streamr_client()` currently consumes an event and forwards its payload to the configured local UDP target without proving that `event.peer` is the configured producer for that client.

Two `Shared=true` Streamr clients using one compatible Yosemite destination/session but different producers can therefore receive each other's application traffic. An unrelated peer that can send to the shared destination must likewise not be treated as the configured producer merely because Yosemite authenticated its identity.

Disposition: **medium security/isolation defect**.

M116 must enforce per-member application routing using the trusted Yosemite-derived remote peer identity.

Required properties:

- an inbound payload is delivered to a Streamr client's local target only when the authenticated peer is proven to be that client's configured producer;
- another shared member's producer traffic is not delivered to this member;
- an unrelated peer is not delivered to any member that did not configure it;
- independent/non-shared Streamr clients retain the same source-authentication invariant;
- no payload-controlled identity, DNS fallback, request-selected LAN routing, or untrusted textual alias may bypass the check;
- subscription/control messages continue to use the configured producer and existing bounded refresh/expiry behavior.

If the configured producer can be represented in multiple textual forms, establish a canonical trusted identity through an existing accepted I2PControl/address-book/Yosemite resolution path before forwarding. Do not build a new router-global resolver. If exact safe producer matching cannot be achieved inside the authorized existing owner, the `Shared × streamrclient` cell must return to `blocked_primitive` and closure must record the exact missing primitive rather than preserving `apply`.

### F5 — `NewDest` semantics are not yet proven by the current runtime effect

M110's client destination store currently generates a fresh private destination whenever `new_dest=true` is staged for a start. It can also combine this with `PersistentClientKey`, with the generated identity winning and potentially becoming persisted.

Read-only Java I2PTunnel evidence maps the corresponding behavior to `i2cp.newDestOnResume`: a new destination is selected when a client session resumes after close-on-idle, and UI/reference logic treats it as dependent on close-on-idle and incompatible with `persistentClientKey`.

Proposal 170 names `NewDest`, `PersistentClientKey`, `Close`, `CloseTime`, and related client lifecycle options but does not, by itself, justify interpreting `NewDest` as unconditional "rotate on every manual start".

M112 still owns the blocked `Close*` client lifecycle family.

Disposition: **medium Proposal/reference conformance defect until resolved**.

M116 must directly freeze the semantics before retaining any `NewDest` cell as `apply`.

The implementation agent must re-read:

- pinned Proposal 170 revision `2026-05-20`;
- the Proposal's cited/reference I2PTunnel option behavior;
- Java I2PTunnel `getNewDest` / `newDestOnResume` behavior at a pinned read-only revision;
- the current M105 evidence and M112 lifecycle ownership.

Only two outcomes are acceptable:

#### Outcome A — standalone Proposal semantics are affirmatively proven

If pinned Proposal/reference evidence establishes a portable `NewDest` trigger that is independent of M112's close-on-idle primitive, implement that exact trigger and prove it in runtime tests. Validate all incompatible option combinations explicitly.

The seven `NewDest` cells may remain `apply` only after that evidence exists.

#### Outcome B — close-on-idle/resume semantics are required

If correct `NewDest` behavior requires `Close`, `CloseTime`, idle closure/resume, or another M112-owned lifecycle primitive, M116 must:

- move the affected `NewDest` cells from `apply` back to `blocked_primitive`;
- make any supplied `NewDest` fail before destination generation/session allocation until the required lifecycle primitive exists;
- remove or make unreachable the M110 "rotate every start" behavior;
- add the cells to M112's residual ownership/count without implementing M112's timer/close framework in M116;
- reject incompatible `NewDest` + `PersistentClientKey` combinations according to the pinned/reference contract where applicable.

Difficulty and desire to keep the matrix green are not evidence for Outcome A.

### F6 — private destination material remains `Debug`-derivable internally

The public `StoredClientDestination` wrapper is redacted, but internal store structs containing raw private keys derive `Debug`.

There is no currently identified log statement that prints those structs, so this is not evidence of an active disclosure. It is nevertheless inconsistent with M110's stronger secret-safety invariant and creates an avoidable future accidental logging surface.

Disposition: **low security hardening defect, required in this corrective**.

Remove raw `Debug` derivation from secret-bearing internal structs or replace it with explicitly redacted implementations. Tests must prove that all intentionally printable secret-store/runtime types redact private material.

## 4. Readiness and ownership decision

M116 is dependency-ready.

All required corrective owners already exist under `emissary-cli/src/i2pcontrol/**`:

- `backends/runtime/session.rs` — shared stream/datagram session registry and compatibility key;
- `backends/runtime/client_listener.rs` — shared stream lease lifetime and cancellation behavior;
- `backends/streamr.rs` — Streamr client producer routing/application isolation;
- `client_secret_store.rs` — client destination transaction and secret redaction;
- `backends/options.rs` — fail-closed common option relationships where needed;
- `production.rs` — existing identity stage/commit/discard transaction, only if needed by the final `NewDest` disposition;
- M095/M105/M110 ledger/tests — current support evidence.

No non-I2PControl production path is required or authorized.

No Yosemite modification, dependency change, core API, util change, startup-manager change, frontend change, workflow change, or router-global owner is required.

If implementation appears to require one of those, stop and return to planning.

## 5. Authorized production paths

M116 may modify only the following production paths when directly required by a work package:

- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs`;
- `emissary-cli/src/i2pcontrol/backends/runtime/client_listener.rs`;
- `emissary-cli/src/i2pcontrol/backends/streamr.rs`;
- `emissary-cli/src/i2pcontrol/client_secret_store.rs`;
- `emissary-cli/src/i2pcontrol/backends/options.rs`;
- `emissary-cli/src/i2pcontrol/production.rs`.

Prefer fewer paths. `production.rs` and `options.rs` are conditional authority, not a requirement to change them.

Focused tests/evidence may change under:

- `emissary-cli/tests/**` only for existing Proposal 170 containment/matrix/live/static-guard suites or a narrowly named M116 regression test if the existing test owners cannot host the case;
- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`;
- `plans/implementation/i2pcontrol-proposal-170/105-residual-option-audit.toml` only for explicit post-M116 reconciliation metadata;
- `plans/implementation/i2pcontrol-proposal-170/110-completion-ledger.toml` only for explicit post-M116 reconciliation metadata;
- M116 plan/closure and active roadmap/registry/README;
- `emissary-cli/tests/m062_dependency_containment.rs` for an exact M116 path helper.

M116 MUST NOT modify:

- `emissary-core/**`;
- `emissary-util/**`;
- `emissary-cli/src/main.rs`;
- `emissary-cli/src/tunnel/**`;
- Cargo manifests or `Cargo.lock`;
- vendored/dependency source;
- frontend/UI paths;
- `.github/**`;
- release paths.

Before production code lands, M062's changed-path guard must authorize the exact M116 paths rather than adding a broad glob.

## 6. Invariants

M116 MUST preserve:

- exact pinned Proposal 170 wire names/types/actions and API 1-only negotiation;
- all 12 canonical tunnel data planes and seven canonical actions;
- M109/M115 startup lifecycle behavior and runtime-disabled isolation;
- M093 anonymity, local-target, HTTP/IRC/Streamr resource, admission, and trusted-peer boundaries;
- no DNS/clearnet fallback for I2P traffic;
- one active generation per control-plane tunnel name;
- bounded cancellation and rollback;
- failed create/edit/start preserves prior durable/running state where the existing transaction promises it;
- no shared-session registry lock across Yosemite network I/O, sleeps, joins, cancellation waits, or filesystem operations;
- no lost waiter and no abandoned creator reservation;
- exact compatibility separation across security/session identities;
- final shared member release tears down the registry-owned session;
- secret material never appears in RPC, errors, logs, ordinary `Debug`, raw config, RouterInfo, or ClientServicesInfo;
- `PrivKeyFile` confinement/copy semantics from M110;
- no startup configuration rewrite;
- no dependency/core/util expansion;
- external interaction remains read-only/internal-only.

## 7. Explicit non-goals

M116 MUST NOT:

- implement M111 `UseSSL`, `TunnelVariance`, `TunnelBackupQuantity`, `SigType`, or `CustomOptions` session-wire semantics;
- implement M112 `Close*`, `Reduce*`, profile, plugin, jump-list, proxy TLS, or timer frameworks merely to keep `NewDest` green;
- implement M113 server presentation/routing/LeaseSet residuals;
- perform M114 final interoperability/reclosure;
- generalize the session registry into router/core ownership;
- add a generic resolver or network identity service;
- change startup M115's neutral shared-session owner;
- change server destination-key ownership except regression verification;
- weaken path confinement to match Java filesystem examples;
- add hosted CI/fuzz/coverage/release infrastructure;
- perform upstream interaction.

## 8. Work packages

### WP1 — freeze the corrective baseline and exact M110 cells

At implementation start:

1. confirm `master` descends from baseline `09247cc...`;
2. re-read M110 plan/closure/ledger and current M095/M105;
3. enumerate the 31 M110 cells and identify which are touched by F1-F6;
4. record the pre-corrective matrix SHA and counts;
5. confirm M111-M114 remain unregistered/blocked;
6. confirm no post-plan commit already corrected a finding.

If master moved materially in these owners, reconcile the plan before editing production code.

### WP2 — make shared acquisition linearizable and cancellation-safe

Refactor the creator/waiter state machine so it has explicit states such as absent/creating/ready and an unambiguous wake/retry protocol.

Required behavior:

- at most one creator performs Yosemite session creation for one compatibility key;
- waiters cannot miss completion notification;
- completion success publishes the owner before waking/releasing waiters;
- completion failure removes/reopens the reservation and wakes waiters;
- creator cancellation/drop removes/reopens the reservation and wakes waiters;
- creator panic or timeout cannot strand `creating=true`;
- a waiter cancellation removes no healthy creator/session and requires no global cleanup;
- last lease release removes the ready session only after the final member is gone;
- capacity accounting counts in-flight reservations predictably and returns bounded errors rather than waiting forever;
- no registry mutex is held through `Session::new`, datagram actor I/O, stream connect, sleeps, or cancellation waits.

Do not rely on timing sleeps as the correctness mechanism.

### WP3 — make compatibility equality secret-safe and collision-safe

Replace the 64-bit private-identity fingerprint as the equality authority.

Required behavior:

- two distinct persistent private identities are incompatible even if all other session fields match;
- equal persistent identities with equal session settings can share when `Shared=true`;
- transient-session compatibility remains explicit and does not accidentally cross style (`stream` vs `datagram`);
- future translated SessionOptions fields are not silently omitted from compatibility;
- nickname exclusion remains documented if intentionally ignored;
- compatibility structures holding secret material have redacted `Debug`/`Display` or are not printable;
- no compatibility key is emitted to logs/errors/RPC.

No new dependency solely for hashing.

### WP4 — enforce Streamr producer isolation

Use the trusted Yosemite `peer` carried by inbound repliable datagrams as the remote identity source.

For each Streamr client generation:

1. establish the configured producer's trusted/canonical identity using an existing accepted owner/path;
2. subscribe/send control as today;
3. before local UDP forwarding, require authenticated inbound peer identity to match the configured producer;
4. drop nonmatching traffic without reflecting payload, peer, or secret data;
5. keep per-client routing independent even when the underlying Yosemite session is shared.

Tests must include at least two clients sharing one session but configured for different producers and an unrelated third peer. Each local UDP target must receive only its configured producer's payloads.

If canonical producer matching cannot be achieved safely with existing I2PControl/Yosemite/address-book primitives, reclassify `Shared × streamrclient` to `blocked_primitive`, fail it before allocation, and record the blocker. Do not add a router-global resolver or accept textual ambiguity.

### WP5 — resolve `NewDest` against pinned/reference semantics

Perform the evidence freeze described in F5 before changing code or matrix.

Record in the M116 closure:

- exact Proposal text relevant to `NewDest`;
- exact reference implementation property and lifecycle trigger;
- relation to `Close`, `CloseTime`, idle close/resume, and `PersistentClientKey`;
- whether the portable contract admits a standalone implementation with current owners.

If Outcome A is proven, implement only that exact semantics and focused lifecycle tests.

If Outcome B is proven, return the affected cells to blocked and transfer them to M112's residual ledger. M116 must not implement M112 timers/idle-close policy.

In either case, remove the unsupported semantic combination where `NewDest` and persistence can silently produce a behavior not sanctioned by the contract.

### WP6 — harden secret Debug surfaces

Audit `client_secret_store.rs` and the M110 session compatibility owner for secret-bearing `Debug`/`Display` implementations.

- remove automatic raw `Debug` from private-key-bearing structs;
- provide redacted implementations only where diagnostic printing is actually needed;
- ensure error strings remain fixed/sanitized;
- preserve owner-only file modes and existing path/symlink/special-file validation;
- add focused redaction tests/static guards where practical.

Do not churn unrelated secret-store code.

### WP7 — reconcile matrix, residual ownership, and evidence ledgers

After runtime/semantic tests pass:

1. update M095 cell-by-cell;
2. calculate exact `apply / blocked_primitive / not_applicable` counts;
3. add explicit post-M116 reconciliation to M110 completion ledger;
4. add explicit post-M116 reconciliation metadata to M105 without rewriting its historical 164-cell input audit;
5. update M112's documented current maximum if `NewDest` cells transfer there;
6. update registry/README/roadmap only at closure to the proven counts;
7. preserve zero accept-inert/unknown/planned cells.

The implementation must not assume the final count in advance.

### WP8 — containment and closure evidence

Update `m062_dependency_containment.rs` with an exact M116 allowlist covering only files actually changed. Do not broaden a glob.

Create `plans/closure/i2pcontrol-proposal-170/116-closure.md` containing:

- implementation head(s);
- requirement-to-evidence table F1-F6;
- exact changed production/test/planning paths;
- deterministic concurrency/cancellation evidence;
- Streamr isolation evidence;
- `NewDest` authority/disposition evidence;
- secret redaction evidence;
- before/after matrix and hashes;
- M105/M110-ledger reconciliation;
- broad verification results;
- security/compatibility/containment review;
- unresolved findings;
- next-handoff decision;
- internal-only attestation.

## 9. Failure, cancellation, restart, and contention model

M116 closure must demonstrate all of the following rather than only happy-path startup:

### Shared creator failure

- first creator returns an error;
- reservation is removed/retryable;
- a later creator succeeds;
- existing unrelated shared sessions are unaffected.

### Shared creator cancellation

- creator is cancelled while Yosemite session creation is pending;
- no stale `creating` entry remains;
- waiter wakes/retries or a later caller can become creator;
- operation completes within the existing bounded start/cancellation contract.

### Lost-wakeup boundary

A deterministic regression must force the historical interleaving where a waiter observes `creating`, the creator publishes completion, and the waiter proceeds after publication. The test must complete without relying on a fortunate scheduler tick.

### Concurrent equivalent starts

Many compatible acquisitions starting concurrently:

- create at most one Yosemite session;
- all successful members receive leases;
- member count remains bounded;
- dropping one lease preserves the session;
- dropping the final lease releases the session;
- a successor acquisition can create a new session cleanly.

### Incompatible identity starts

Distinct persistent identities never share even with otherwise identical session settings.

### Streamr shared routing

One shared datagram session with multiple producer-specific members must not cross-deliver.

### Identity transaction

Failed key generation/import/session allocation still discards staged identity; commit failure still stops the newly started backend as M110 intended.

### `NewDest`

Tests must exercise the actual accepted trigger, not merely parser/storage presence. If blocked, tests must prove fail-before-allocation.

## 10. Focused tests

At minimum add or strengthen tests for:

1. waiter cannot miss creator success notification;
2. waiter cannot miss creator failure notification;
3. creator cancellation releases the reservation;
4. concurrent compatible stream acquisition creates one session;
5. concurrent compatible datagram acquisition creates one session;
6. final member release tears down stream/datagram registry entries;
7. different persistent identities never share;
8. compatibility logging/debugging cannot expose private identity;
9. Streamr A/B shared-session traffic is delivered only to the matching configured producer target;
10. unrelated Streamr peer traffic is dropped;
11. Streamr cancellation/final lease release remains bounded;
12. secret-store printable types redact or cannot print raw private keys;
13. `NewDest` exact accepted trigger or fail-before-allocation blocked behavior;
14. invalid `NewDest`/`PersistentClientKey` combinations fail deterministically where the pinned/reference contract requires;
15. matrix tests enforce exact post-M116 counts and dispositions;
16. M062 containment rejects any changed production path outside M116 authority.

Use fake SAM/reference fixtures already present where possible. Do not add a general mocking framework solely for this pass.

## 11. Broad verification

Run and record at minimum:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Also run any M116-specific regression target added under WP10.

The repository's known stable/nightly rustfmt mismatch remains a tooling limitation. Record it accurately; do not retain unrelated formatting churn.

Do not add hosted CI loops or broad verification infrastructure.

## 12. Acceptance criteria

M116 closes only if all are true:

1. no shared-session waiter can sleep indefinitely because of a lost `Notify` wakeup;
2. cancellation/drop/failure of the active creator cannot strand a compatibility key in `creating`;
3. equivalent concurrent acquisitions create one bounded session owner;
4. persistent identity compatibility is collision-safe and secret-safe;
5. shared Streamr members cannot receive another producer's payload, and unrelated-peer payloads are not forwarded;
6. `NewDest` has a directly evidenced pinned/reference disposition;
7. every `NewDest` cell marked `apply` has the exact required runtime trigger; otherwise it is returned to `blocked_primitive` and assigned to the correct future owner;
8. unsupported `NewDest` combinations fail before identity/session allocation;
9. secret-bearing internal store/session types do not expose raw private material through ordinary `Debug`/`Display`;
10. M095/M105/M110 ledger are reconciled to one exact current cell count with zero accept-inert/unknown/planned cells;
11. M061/M062/M093 and M109/M115 regressions remain satisfied;
12. no non-I2PControl production path, Cargo/dependency, core/util, startup, frontend, workflow, or release path changed;
13. broad verification is recorded with no unexplained failure;
14. closure explicitly decides whether any successor is dependency-ready.

## 13. Stop conditions

Stop and return to planning rather than widening if:

- fixing shared acquisition requires changing Tokio/Yosemite or adding a dependency rather than correcting the local owner;
- correct Streamr producer identity matching requires a new router-global resolver/core API;
- `NewDest` requires M112's close-on-idle/timer framework;
- a safe correction requires modifying `emissary-core`, `emissary-util`, startup tunnel owners, Cargo manifests/lockfile, or vendored dependency source;
- any secret would need to enter raw config, logs, RPC, or ordinary printable state;
- a matrix cell can remain `apply` only by accepting an inert/approximate semantic.

For the `NewDest`/Streamr cases, a stop condition does not mean M116 cannot close: it means the affected cell must be truthfully returned to `blocked_primitive` with a named future owner/blocker, while the safe M116 corrections still close.

## 14. Successor disposition

M111-M114 remain blocked throughout M116 implementation.

M116 closure must reassess but must not automatically register M111 simply because M116 closes.

Expected post-M116 state:

- M111 remains dependency-blocked unless an independently accepted Yosemite public session-wire capability appeared;
- M112 remains blocked and may gain the seven `NewDest` cells if close-on-idle/resume semantics are required;
- M113 remains blocked on presentation/LeaseSet primitives;
- M114 remains blocked until zero applicable residual cells and no open medium/high corrective remain.

If no successor is dependency-ready, registry `Current handoff` returns to `none` after M116 closure.

## 15. Internal-only boundary

All work remains internal to the `eggstack/emissary` fork.

External Proposal/reference/Tokio/Yosemite material is read-only evidence. No upstream issue, pull request, review request, submission, merge/adoption request, branch/tag push, release, maintainer contact, contribution preparation, or external repository mutation is authorized.
