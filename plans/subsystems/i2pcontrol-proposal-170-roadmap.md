# I2PControl Proposal 170 Operational Corrective Roadmap

Status: partial Proposal 170 support; M035 ready

Planning baseline:

- `5a2e216` — M033 implementation/test head

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
- `plans/implementation/i2pcontrol-proposal-170/README.md`

## 1. Purpose

M030 closed the demonstrated AddressBook destination-owner defect and retained an
honest `partial Proposal 170 support` disposition. The next workstream converts
that partial implementation into a more operational administrative API without
expanding into missing tunnel families, broad router-core telemetry, frontend
work, or upstream contribution activity.

The primary capability goal is real TunnelManager lifecycle support for generic
`client` and `server` tunnels, the two Proposal 170 types whose data planes
already exist in Emissary. The work must preserve the security-audited router
core and keep Proposal 170 policy, persistence, supervision, and wire behavior
inside `emissary-cli/src/i2pcontrol/**` wherever possible.

The secondary goals correct remaining truthful-operation, compatibility,
security, persistence, and containment findings:

- AddressBook `SetSubscriptions` and `SetConfig` must not report operational
  success for inert metadata;
- base I2PControl compatibility and overlapping RouterInfo names require an
  explicit tested boundary;
- authentication comparison/throttling and publication durability require
  focused hardening;
- existing Proposal 170-specific SAM and AddressBook policy outside the
  I2PControl boundary should be reduced only where that can be done without a
  broad refactor;
- one bounded live-runtime interoperability exercise must verify the composed
  service rather than relying only on handler/fake-adapter tests.

The expected end state remains partial support unless every unavailable
RouterInfo source and unsupported tunnel family is separately implemented and
evidenced. This roadmap does not authorize that expansion.

## 2. Current state

### 2.1 Retained closed dimensions

Do not reimplement these without a new direct defect:

- standard authentication/token/error and JSON-RPC request-ID/notification
  behavior from M020;
- exact Proposal 170 wire names, casing, direct-presence semantics, action and
  type inventory;
- atomic tunnel-definition persistence, secret response filtering, and
  exhaustive backend registration from M021;
- startup tunnel inventory and ClientServicesInfo listener/source behavior from
  M023;
- bounded SAM observation behavior from M024;
- exact 43-selector RouterInfo contract and 16 available / 1 neutral / 26
  unavailable source classification from M025/M026;
- literal wire fixtures from M027;
- AddressBook compile-time/runtime isolation from M028;
- enabled AddressBook destination and lookup owner coherence from M030;
- internal-only/no-upstream governance.

### 2.2 Remaining material findings

1. M031's production backend registry maps generic `client` to a real backend
   and the other eleven tunnel types to unsupported backends. Definition CRUD
   and generic client lifecycle are real; generic server lifecycle is not yet
   operational.
2. The startup generic client/server managers remain startup-oriented and do
   not expose safe administrative adoption; M031 adds named cancellation only
   for independently owned control-plane clients.
3. AddressBook subscription/config setters persist metadata but do not control
   the active downloader/configuration, while returning success.
4. The protected dispatcher is a Proposal-170-focused subset of the base
   I2PControl API; overlapping selector names may not preserve historical
   behavior.
5. Authentication uses a hand-written comparison and has no bounded failed-login
   throttling.
6. Persistent publication syncs files but not all containing directories, so
   strict power-loss durability is stronger in documentation than in evidence.
7. Proposal 170-specific AddressBook policy remains substantial in the original
   CLI AddressBook module, and SAM observation aggregation materially changes
   core SAM files.
8. GitHub has no attached CI status for the current head; retained local test
   evidence is extensive but not a live composed-router interoperability run.

## 3. Scope boundary

### 3.1 In scope

- real `client` and `server` TunnelManager backends using existing Emissary data
  planes;
- an I2PControl-owned per-name runtime supervisor for control-plane definitions;
- safe start, stop, restart, inspect, failure recovery, delete, rename, and
  `StartOnLoad` behavior for eligible types;
- a backend-owned, path-confined server destination identity store;
- truthful AddressBook subscription/config operation results;
- base compatibility and overlapping-selector reconciliation;
- bounded authentication and publication durability hardening;
- reduction of Proposal 170 policy outside the I2PControl boundary where
  behavior-preserving and reviewable;
- focused local and one bounded live-runtime interoperability test;
- directly affected documentation, static guards, implementation dispositions,
  and independent closure.

### 3.2 Explicitly out of scope

- new HTTP client/server, bidirectional HTTP server, IRC client/server,
  SOCKS-IRC, CONNECT, Streamr client/server, or other tunnel data planes;
- treating the existing HTTP/SOCKS startup proxies as Proposal 170 I2PTunnel
  backends without a separate ownership and contract plan;
- adoption, mutation, stop, restart, or deletion of startup-managed tunnel tasks;
- new RouterInfo sources, rolling samplers, polling loops, NetDB inspection,
  peer classification, queue instrumentation, or fabricated values;
- router, transport, streaming protocol, LeaseSet, cryptographic, routing, or
  tunnel-building algorithm changes;
- frontend/UI controls;
- a repository-wide crate extraction or general service framework;
- schema redesign unrelated to the exact backend or durability requirement;
- remote CI matrices, release/publishing automation, coverage gates, fuzz
  campaigns, soak farms, generated evidence bundles, or unrelated test debt;
- upstream issues, pull requests, reviews, discussions, submissions, adoption,
  merge solicitation, maintainer outreach, or contribution preparation.

## 4. Target architecture

### 4.1 Administrative layer

JSON-RPC handlers, canonical/compatibility parsing, domain definitions,
persistence, operation-status translation, and validation remain inside
`emissary-cli/src/i2pcontrol/**`.

Handlers continue to depend only on `TunnelManagerControl`; they do not bind
listeners, spawn data-plane tasks, manipulate key files, or know backend-specific
configuration.

### 4.2 Runtime supervisor

Add an I2PControl-owned control-plane runtime supervisor:

- keyed by canonical tunnel name;
- bounded by the existing tunnel inventory limit;
- stores only task/cancellation/state metadata required for control-plane-owned
  instances;
- serializes transitions per name without globally serializing unrelated
  tunnels;
- removes completed tasks and permits corrected definitions to restart;
- never owns startup-managed tasks;
- does not hold the TunnelStore lock across runtime operations.

### 4.3 Existing data-plane adapters

The original CLI tunnel modules remain the canonical data-plane code. Narrow
single-instance runners may be extracted:

- client: bind one configured listener, create/connect the existing Yosemite
  stream session, copy bidirectionally, retry according to bounded policy, and
  terminate on cancellation;
- server: create/load a backend-owned destination, create the existing Yosemite
  persistent session, publish destination metadata, issue forwarding, and
  terminate on cancellation.

Those adapters must not import I2PControl domain, store, handler, or JSON-RPC
types. Translation remains in I2PControl.

### 4.4 Server secret identity

Control-plane server destinations live under a fixed I2PControl-owned state
directory. The backend accepts no arbitrary key path. Identity is durable across
stop/restart and rename, permissions are restrictive where supported, and key
material is omitted from logs and responses.

### 4.5 Startup ownership

Startup configuration remains authoritative for existing startup tasks.
I2PControl may observe their definitions and actual destination metadata but
must reject lifecycle and mutation. There is no adoption or dual authority.

### 4.6 AddressBook operation truthfulness

`SetSubscriptions` and `SetConfig` must have one of two outcomes:

- a supported field is applied to the actual runtime owner and the response
  reports success only after the operational state accepts it; or
- the request receives a deterministic unsupported/error result before inert
  metadata is published.

The implementation must not add arbitrary path control merely because Proposal
170 lists Java address-book file paths. Existing path ownership remains fixed by
Emissary configuration.

### 4.7 Compatibility boundary

Direct Proposal 170 selectors and historical nested/base requests remain
separate request modes. Exact-name overlaps receive mode-specific behavior and
fixtures. Unsupported base methods are documented and must not be silently
represented as implemented.

## 5. Dependency graph

```text
M030 retained partial-support closure
                |
                v
M031 runtime supervisor + generic client backend
                |
                v
M032 generic server backend + secret identity
                |
                v
M033 lifecycle recovery, StartOnLoad, and TunnelManager closure
                |
                v
M034 AddressBook setter truthfulness
                |
                v
M035 base compatibility and selector overlap
                |
                v
M036 auth and persistence durability hardening
                |
                v
M037 I2PControl containment reduction
                |
                v
M038 live-runtime interoperability validation
                |
                v
M039 independent final-head operational reclosure
```

M035 is the only dependency-ready implementation handoff. Later plans are
written for handoff continuity but remain blocked until their named hard
dependencies close. This prevents parallel edits to the same lifecycle and
composition seams.

## 6. Milestones

### M031 — Runtime supervisor and generic client backend

Status: closed

Plan:

- `plans/implementation/i2pcontrol-proposal-170/031-client-tunnel-runtime-backend.md`

Objective:

- establish the control-plane runtime supervisor;
- extract the smallest reusable single-client runtime primitive;
- replace only the `client` unsupported backend with a real backend;
- prove isolated start/stop/restart/failure behavior without touching core.

Exit conditions:

- control-plane `client` definitions can start and stop truthfully;
- startup-managed clients remain external;
- unrelated unsupported types remain resource-free;
- no `emissary-core/**` production change;
- every external CLI file change is justified;
- implementation disposition freezes the head.

### M032 — Generic server backend and destination identity

Status: closed

Plan:

- `plans/implementation/i2pcontrol-proposal-170/032-server-tunnel-runtime-backend.md`

Objective:

- reuse the existing generic server runtime through a narrow adapter;
- replace only the `server` unsupported backend;
- implement backend-owned persistent destination identity and rename/delete
  semantics;
- prove no arbitrary key path or secret disclosure.

### M033 — Lifecycle reconciliation and StartOnLoad

Status: closed

Plan:

- `plans/implementation/i2pcontrol-proposal-170/033-tunnel-lifecycle-reconciliation.md`

Objective:

- reconcile runtime and durable definition state;
- implement deterministic `StartOnLoad` for eligible control-plane definitions;
- close concurrent lifecycle, restart, delete, rename, and failure-recovery
  semantics;
- revalidate all twelve backend registrations and public status shapes.

### M034 — AddressBook setter truthfulness

Status: closed

Plan:

- `plans/implementation/i2pcontrol-proposal-170/034-addressbook-setter-truthfulness.md`

Objective:

- stop successful responses for inert subscription/config metadata;
- implement only safely enforceable runtime fields;
- reject path-based or unsupported settings before persistence;
- keep M028/M030 owner and disabled-mode invariants.

### M035 — Base compatibility and selector overlap

Status: ready

Plan:

- `plans/implementation/i2pcontrol-proposal-170/035-base-compatibility-and-selector-overlap.md`

Objective:

- freeze the exact supported base API inventory;
- correct mode-specific behavior for overlapping RouterInfo keys;
- preserve historical nested requests where implemented;
- document unsupported base methods without inventing capability claims.

### M036 — Authentication and publication durability hardening

Status: blocked on M035

Plan:

- `plans/implementation/i2pcontrol-proposal-170/036-auth-and-publication-hardening.md`

Objective:

- replace the hand-written password comparator with a reviewed primitive;
- add bounded failed-authentication throttling without global denial of service;
- qualify or strengthen file/directory synchronization and recovery semantics for
  affected I2PControl stores;
- preserve local-only defaults and existing wire errors.

### M037 — Containment reduction and static boundary

Status: blocked on M036

Plan:

- `plans/implementation/i2pcontrol-proposal-170/037-containment-boundary-reduction.md`

Objective:

- reduce Proposal 170 policy outside `i2pcontrol/**`;
- leave only narrow runtime hooks in the original CLI modules;
- reduce SAM core changes toward an optional passive observer seam where
  behavior-preserving;
- add static changed-path and dependency guards;
- stop rather than perform a broad crate/core refactor.

### M038 — Live-runtime interoperability validation

Status: blocked on M031–M037

Plan:

- `plans/implementation/i2pcontrol-proposal-170/038-live-runtime-interoperability.md`

Objective:

- run one bounded local Emissary instance with I2PControl enabled;
- exercise TLS authentication, AddressBook entry behavior, available RouterInfo,
  ClientServicesInfo, TunnelManager CRUD, real client/server lifecycle, and
  explicit unsupported paths;
- capture reproducible local evidence without new CI infrastructure.

### M039 — Independent operational reclosure

Status: blocked on M038

Plan:

- `plans/implementation/i2pcontrol-proposal-170/039-operational-reclosure.md`

Objective:

- independently review the actual final head;
- classify wire, source, runtime, persistence, feature isolation, containment,
  security, and evidence dimensions;
- select `partial Proposal 170 support`, `corrective pass required`, or `blocked`
  without overstating unsupported selectors or tunnel families.

## 7. Cross-cutting invariants

1. Proposal 170 names, casing, JSON types, presence semantics, and status channels
   remain exact.
2. Existing compatibility aliases are not expanded.
3. Startup-managed tunnels remain externally owned and read-only.
4. Only `client` and `server` receive real backends under this roadmap.
5. Unsupported tunnel types open no listener/session/task and never report
   running.
6. Runtime failure is isolated to one control-plane tunnel and does not require
   router restart or store deletion to recover.
7. No persistence lock is held across network I/O, bind, cancellation, sleep, or
   join.
8. Stop targets one exact named runtime; restart completes stop before start.
9. Server private destination material remains internal and path-confined.
10. Disabled/default I2PControl behavior remains unaffected.
11. AddressBook owner coherence from M030 remains intact.
12. Unavailable RouterInfo values remain explicit and unfabricated.
13. No `emissary-core/**` change is authorized by M031–M036.
14. M037 may only reduce existing core coupling through a narrow passive seam;
    it may not add router behavior.
15. Every production file outside `i2pcontrol/**` requires an explicit
    requirement and changed-path justification.
16. No frontend, CI/release expansion, unrelated refactor, or upstream activity
    is authorized.

## 8. Failure, cancellation, restart, and contention policy

### Lifecycle

- Start validates and reserves the name before task allocation.
- Bind/SAM/startup failure publishes failed/stopped state and releases the name.
- Stop is idempotent for absent/stopped tasks and awaits bounded cancellation.
- Restart is stop-then-start; it never permits old and new tasks to overlap.
- Panic/completion removes the task handle and records sanitized failure.
- Delete of a running eligible tunnel must stop it first or fail without deleting
  durable state.
- Rename of a running tunnel is rejected unless the milestone proves a fully
  atomic stop/rename/start sequence; silent split identity is prohibited.

### Persistence

- Definition and secret mutations validate before publication.
- Failure leaves the previous generation and runtime task coherent.
- `StartOnLoad` failure does not block unrelated tunnels or falsely report
  running.
- Server secret rename/delete is coordinated with definition mutation.
- Directory durability is either implemented and tested or documentation is
  narrowed to the actual guarantee.

### Contention

- Per-name lifecycle transitions are serialized.
- Unrelated tunnel names may progress concurrently within global bounds.
- `All` operations remain bounded and deterministic.
- Authentication throttling is bounded per source/context and cannot consume
  unbounded memory.

## 9. Compatibility and migration

- Existing startup configurations remain valid and unchanged.
- Existing control-plane definition generations remain readable.
- Existing `client` and `server` definitions become operational without public
  schema migration.
- Unsupported definitions remain stored and inactive.
- Server runtime identity receives an internal migration only when needed; no
  arbitrary user path is introduced.
- AddressBook state from M030 remains readable without a new authority.
- Existing direct Proposal 170 requests retain their current exact wire.
- Compatibility behavior is corrected through request-mode distinction rather
  than new aliases.

## 10. Verification policy

Use local, package-scoped verification. Each milestone runs focused tests first,
then the smallest broad matrix that covers changed paths. The normal upper bound
is:

```bash
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

When a milestone changes an original non-I2PControl CLI module, also run its
focused no-feature tests to prove default behavior remains unchanged. M037 may
run focused core SAM tests only because it is explicitly a coupling-reduction
milestone.

Use targeted formatting and `git diff --check`. Do not add remote CI, platform
matrices, coverage gates, fuzz/soak farms, release checks, or generated evidence
bundles.

## 11. Documentation and closure discipline

Each implementation milestone must create an implementation disposition under
`plans/closure/i2pcontrol-proposal-170/` containing:

- implementation commits and frozen head;
- exact changed production/test/documentation files;
- justification for every file outside `i2pcontrol/**`;
- requirement-to-evidence matrix;
- exact command outcomes;
- failure/cancellation/restart/contention review;
- compatibility and migration review;
- security and secret-handling review;
- unresolved findings with severity;
- no-upstream attestation.

A coding commit or test count is not closure. M039 remains the distinct final
review.

## 12. Milestone status

| Milestone | Status | Disposition |
|---|---|---|
| 001–019A | historical/superseded/invalidated as recorded | retained history |
| 020–030 | retained closed evidence | current partial-support baseline |
| 031 | closed | runtime supervisor and generic client backend |
| 032 | closed | generic server backend and persistent destination identity |
| 033 | closed | lifecycle reconciliation and StartOnLoad |
| 034 | closed | AddressBook setter truthfulness |
| 035 | ready | hard dependency M034 closed |
| 036 | blocked | hard dependency M035 |
| 037 | blocked | hard dependency M036 |
| 038 | blocked | hard dependencies M031–M037 |
| 039 | blocked | hard dependency M038 |

## 13. Completion definition

This roadmap is complete only when:

- real generic client and server backends are operational and isolated;
- startup-managed ownership remains unchanged;
- `StartOnLoad`, stop/restart/delete/rename, failure, and recovery semantics are
  coherent;
- inert AddressBook setter success is eliminated;
- base compatibility overlap is tested and documented;
- authentication and persistence claims match implemented guarantees;
- Proposal 170-specific coupling outside the I2PControl boundary is minimized
  without broad refactoring;
- one live-runtime interoperability run passes or records a precise blocker;
- M039 independently reviews the final head;
- no unresolved high/medium correctness, security, compatibility, ownership, or
  scope defect remains in the implemented dimensions;
- unavailable RouterInfo sources and unsupported tunnel families remain
  explicit rather than fabricated;
- no upstream interaction occurred.

The expected honest final label is still `partial Proposal 170 support` unless
separately authorized future work closes every unavailable source/runtime.
