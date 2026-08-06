# I2PControl Proposal 170 Operational Corrective Roadmap

Status: partial Proposal 170 support; corrective sequence closed

Planning baseline:

- `75514ce` — corrected final evidence head reviewed by M044

Pinned external authority:

- I2P Proposal 170, `I2PControl Expansion`
- status: `Open`
- created and last updated: `2026-05-20`
- existing I2PControl authentication and JSON-RPC contract

Canonical internal references:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`
- `plans/closure/i2pcontrol-proposal-170/030-closure.md`
- `plans/closure/i2pcontrol-proposal-170/039-closure-invalidation.md`
- `plans/implementation/i2pcontrol-proposal-170/README.md`

## 1. Purpose

M031–M039 made the Proposal 170 service materially operational while retaining
an honest partial-support boundary. Generic control-plane `client` and `server`
tunnels gained real backends, lifecycle reconciliation, fixed server identity,
truthful AddressBook behavior, compatibility inventories, authentication and
publication hardening, containment reduction, and bounded live validation.

A final-head review after M039 demonstrated three defects that the accepted
evidence did not cover:

1. the startup-managed generic server manager drops its watch sender before
   entering the reusable server runtime, allowing immediate self-cancellation;
2. failed-authentication throttling is keyed by full source socket address and
   can be bypassed by reconnecting with a new ephemeral port; its split
   read/sleep/write accounting also undercounts concurrent bursts;
3. `SetSubscriptions` can durably commit a replacement and then return failure
   if refresh scheduling becomes unavailable.

The M039 final disposition is therefore invalidated. M040–M044 correct only
these demonstrated defects, add the missing exact-path evidence, and perform a
new independent final-head review.

The expected end state remains `partial Proposal 170 support`. This roadmap does
not authorize the ten missing tunnel data planes or 26 unavailable RouterInfo
sources.

## 2. Retained implementation state

Unless a corrective milestone demonstrates a direct additional defect, retain:

- standard I2PControl authentication/token/error and JSON-RPC ID/notification
  behavior;
- exact Proposal 170 names, casing, types, selector presence semantics, tunnel
  inventory, and action inventory;
- real control-plane generic client and server backends;
- per-name lifecycle supervision, generation fencing, stop-before-restart,
  delete/edit/rename coordination, and eligible `StartOnLoad`;
- backend-owned fixed-path server destination identity and secret redaction;
- startup/control-plane ownership separation and administrative rejection of
  startup-managed mutation;
- resource-free explicit unsupported status for ten tunnel families;
- AddressBook entry ownership, full-destination coherence, feature isolation,
  live subscription control, and explicit non-empty `SetConfig` rejection;
- direct/base compatibility inventories and overlap handling;
- reviewed constant-time password comparison;
- bounded TLS/request/connection/concurrency limits;
- atomic publication, backup recovery, restrictive permissions, path
  confinement, and directory-sync qualification;
- bounded passive SAM observation with Proposal 170 aggregation in
  `emissary-cli/src/i2pcontrol/**`;
- RouterInfo classification of 16 available, 1 neutral, and 26 unavailable
  additions;
- internal-only/no-upstream governance.

The corrective sequence must not reimplement these dimensions.

## 3. Current corrective findings

### 3.1 Startup server cancellation ownership

The original startup `ServerTunnelManager` creates a watch channel while
discarding its sender. The reusable runtime treats sender loss as cancellation.
This is a high-severity regression in existing startup server functionality and
contradicts the M032/M039 behavior-preservation claim.

### 3.2 Authentication source identity and atomic accounting

The throttle map uses `SocketAddr`, so source-port churn resets accumulated
state. Delay is computed before failure count is recorded, so concurrent invalid
attempts can reserve the same stale count. This is a medium security defect in
the M036/M039 throttle claim.

### 3.3 AddressBook mutation linearization

The live manager commits subscriptions before refresh scheduling. A failed
post-commit enqueue can produce an error response for an already-applied
mutation. This is a medium operation-truthfulness defect in the M034/M039 claim.

### 3.4 Evidence gap

M038 exercises a startup client but not the original startup server manager.
Authentication tests omit same-IP/different-port and concurrent reservation.
AddressBook tests omit refresh-worker failure after durable commit.

## 4. Scope boundary

### 4.1 In scope

- retaining the startup server runtime's cancellation sender for its lifetime;
- direct startup-manager regression against a bounded fake SAM endpoint;
- normalizing authentication throttle identity to source IP;
- atomically reserving failed-auth counts before sleeping;
- defining one durable `SetSubscriptions` linearization point;
- making refresh scheduling bounded follow-up work that cannot reverse a
  completed mutation result;
- focused regressions for the exact missed paths;
- the existing package-scoped verification matrix;
- closure invalidation, dispositions, documentation, and independent reclosure.

### 4.2 Explicitly out of scope

- HTTP client/server, bidirectional HTTP server, IRC client/server, SOCKS-IRC,
  CONNECT, Streamr, or any other missing tunnel data plane;
- control, adoption, restart, stop, or deletion of startup-managed tasks;
- new RouterInfo sources, polling, rolling samplers, NetDB inspection, or
  fabricated values;
- router, transport, streaming protocol, LeaseSet, cryptographic, routing, or
  tunnel-building changes;
- frontend/UI work;
- a repository-wide crate extraction or service framework;
- persistent accounts, distributed bans, proxy-header trust, or firewall
  integration;
- a new AddressBook scheduler, event bus, second authority, arbitrary paths, or
  synchronous download transaction;
- remote CI matrices, release automation, coverage gates, fuzz campaigns, soak
  farms, generated evidence bundles, or unrelated test cleanup;
- upstream issues, pull requests, reviews, discussions, submissions, adoption,
  merge solicitation, maintainer outreach, or contribution preparation.

## 5. Ownership and target architecture

### 5.1 Startup server runtime

The original CLI server manager remains the owner of startup-configured server
tasks. It retains a local watch sender solely to keep the reusable runtime's
cancellation channel open. The sender is not exposed through I2PControl and does
not become an administrative stop handle.

Production work is limited to `emissary-cli/src/tunnel/server.rs`.

### 5.2 Authentication throttle

The I2PControl server obtains a peer `SocketAddr` at the TLS/HTTP boundary and
normalizes it to `IpAddr` for throttle state. One short locked operation expires
old state, reserves the next count, enforces capacity, and returns a bounded
delay. The lock is released before sleep.

No forwarded header is trusted. State remains in-memory and process-local.

### 5.3 AddressBook subscription mutation

The live AddressBook manager remains the sole runtime owner. `SetSubscriptions`
success means validation, durable owner publication, and active-set replacement.
Remote download completion is not part of the mutation transaction.

Refresh scheduling remains bounded follow-up work. A post-commit scheduling
failure may produce a sanitized internal diagnostic but must not turn the
completed mutation into an error response.

### 5.4 Validation and closure

M043 validates exact corrected paths and the retained matrix without production
patches. M044 independently reviews the final head and selects the truthful
status.

## 6. Dependency graph

```text
M039 closure invalidated
          |
          v
M040 startup server cancellation-owner correction
          |
          v
M041 auth throttle IP identity + atomic reservation
          |
          v
M042 AddressBook subscription commit boundary
          |
          v
M043 corrective runtime regression validation
          |
          v
M044 independent corrective final-head reclosure
```

M040–M044 are closed. No successor is registered or dependency-ready: deferred
RouterInfo sources and unsupported tunnel families remain outside this roadmap.

## 7. Corrective milestones

### M040 — Startup server cancellation-owner correction

Status: closed

Plan:

- `plans/implementation/i2pcontrol-proposal-170/040-startup-server-cancellation-correction.md`

Objective:

- retain the startup server watch sender for the runtime lifetime;
- prove the original manager reaches HELLO, SESSION CREATE, destination
  observation, and STREAM FORWARD;
- preserve startup ownership and control-plane behavior;
- change no core or protocol code.

Exit conditions:

- the regression fails on `563e093` and passes after correction;
- the startup runtime remains alive after readiness;
- no administrative startup-task handle is added;
- only the authorized server tunnel module and focused tests change;
- M040 closure is accepted.

### M041 — Authentication throttle source/accounting correction

Status: closed

Plan:

- `plans/implementation/i2pcontrol-proposal-170/041-auth-throttle-source-accounting.md`

Objective:

- key failure state by normalized source IP;
- reserve/increment failure count atomically before delay;
- preserve bounded capacity, monotonic capped delays, exact errors, tokens, and
  constant-time comparison.

### M042 — AddressBook subscription commit-boundary correction

Status: closed

Plan:

- `plans/implementation/i2pcontrol-proposal-170/042-addressbook-subscription-commit-boundary.md`

Objective:

- define one durable linearization point;
- prohibit failure responses after commit;
- preserve pre-commit failure rollback and bounded refresh coalescing;
- retain explicit unsupported `SetConfig` and owner/feature isolation.

### M043 — Corrective runtime regression validation

Status: closed

Plan:

- `plans/implementation/i2pcontrol-proposal-170/043-corrective-runtime-regression-validation.md`

Objective:

- exercise the original startup server path;
- exercise ephemeral-port and concurrent auth cases;
- exercise post-commit refresh-worker failure;
- run the retained bounded verification matrix;
- make no production changes.

### M044 — Corrective final-head reclosure

Status: closed; partial Proposal 170 support

Plan:

- `plans/implementation/i2pcontrol-proposal-170/044-corrective-final-head-reclosure.md`

Objective:

- independently review the exact final head;
- verify the corrected and retained dimensions;
- select `partial Proposal 170 support`, `corrective pass required`, or
  `blocked` without production changes.

## 8. Historical milestones and evidence

| Milestone | Current disposition |
|---|---|
| M020–M030 | retained component evidence; M030 is the controlling pre-runtime baseline |
| M031 | retained generic client backend/supervisor evidence |
| M032 | retained server backend/identity evidence; startup behavior-preservation claim invalidated |
| M033 | retained lifecycle/StartOnLoad evidence |
| M034 | retained live setter/config evidence; post-commit result claim invalidated |
| M035 | retained compatibility evidence |
| M036 | retained password/publication evidence; throttle effectiveness claim invalidated |
| M037 | retained containment evidence subject to startup adapter correction |
| M038 | retained live evidence; exact-path coverage gap recorded |
| M039 | final disposition invalidated by `039-closure-invalidation.md` |

Historical records remain in place and are not rewritten or deleted.

## 9. Cross-cutting invariants

1. Proposal 170 names, casing, JSON types, presence semantics, and status channels
   remain exact.
2. Existing compatibility aliases are not expanded.
3. Startup-managed tunnels remain externally owned and read-only.
4. Only generic `client` and `server` have real backends under this roadmap.
5. Unsupported tunnel types allocate no listener/session/task and never report
   running.
6. No persistence lock is held across network I/O, sleep, cancellation, or join.
7. Server private destination material remains fixed-path, internal, and
   redacted.
8. Disabled/default I2PControl behavior remains unaffected.
9. AddressBook owner coherence and full-destination precedence remain intact.
10. Unavailable RouterInfo values remain explicit and unfabricated.
11. The auth throttle remains bounded, local, and non-persistent.
12. No post-commit AddressBook mutation failure is reported.
13. No new `emissary-core/**` behavior is authorized.
14. Every production file outside `i2pcontrol/**` requires an explicit defect
    and changed-path justification.
15. No frontend, CI/release expansion, broad refactor, or upstream activity is
    authorized.

## 10. Failure, cancellation, restart, and contention policy

### Startup server

- The local sender keeps the cancellation channel open.
- Startup task termination remains process/task-owner controlled.
- No new administrative cancellation is introduced.
- SAM/session/forward failures retain existing bounded handling.

### Authentication

- Invalid-password failure is reserved under the short throttle lock.
- Cancellation during delay leaves the failure recorded.
- Success clears the normalized source IP.
- Capacity and delay remain bounded.

### AddressBook

- Validation and publication failure occur before commit and preserve prior
  state.
- After durable commit, refresh failure cannot reverse or relabel the mutation.
- Refresh work remains bounded and coalesced.
- Network failure never rolls back the active subscription set.

### Validation

- Tests use loopback, ephemeral ports, bounded timeouts, and temporary state.
- A material production defect discovered in M043/M044 requires a new plan.

## 11. Verification policy

Use focused tests first, then the existing bounded matrix:

```bash
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features
cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings

cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings

cargo check -p emissary-core
cargo test -p emissary-core sam
cargo clippy -p emissary-core --all-targets -- -D warnings

git diff --check
```

Each milestone adds exact focused commands. Use targeted formatting because the
repository's stable/nightly rustfmt mismatch is known. Do not add remote CI,
release, coverage, fuzz, soak, or network-farm infrastructure.

## 12. Risks and mitigations

| Risk | Mitigation |
|---|---|
| A minimal sender fix grows into startup task control | Keep sender local and unexposed; changed-path review |
| IP throttling creates global lock contention | Short in-memory reservation only; no lock across sleep |
| Concurrent failures are undercounted | Reserve count atomically before delay |
| AddressBook response remains ambiguous | Test worker failure after durable commit and pre-commit failure separately |
| Corrective tests use the wrong backend | Require the original `ServerTunnelManager` path |
| Scope expands into missing capability work | Static plan guards and M044 changed-path classification |
| Closure repeats M039 overclaim | Independent M044 matrix and explicit unavailable capability status |

## 13. Final status rule

M044 may select `partial Proposal 170 support` only when every implemented and
claimed dimension is exact, operational, bounded, and evidenced.

Ten tunnel families remain unsupported and 26 RouterInfo additions remain
unavailable. Full Proposal 170 completion and `closed internally against pinned
revision` are not available under this roadmap.

Any unresolved high/medium defect requires `corrective pass required`. Missing
or unreviewable evidence requires `blocked`.

No status implies upstream review, acceptance, certification, adoption, or
merge.
