# M038 — Live-Runtime Proposal 170 Interoperability Validation

Status: blocked on M031–M037

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Applicable governance:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`

Repository baseline:

- accepted M037 implementation/closure head, to be recorded before execution

Hard dependencies:

- M031 through M037 closed

## 1. Bounded objective

Validate the composed, feature-enabled I2PControl service against one real local
Emissary runtime rather than relying only on handler, fake-adapter, and store
unit tests.

The validation must exercise TLS/authentication, AddressBook entry and
subscription behavior, available/unavailable RouterInfo selectors,
ClientServicesInfo, TunnelManager CRUD, real generic client/server lifecycle,
unsupported tunnel behavior, restart, and failure recovery.

M038 is evidence and small test-harness work only. It does not add production
capability, remote CI, privileged containers, network farms, Java I2P/i2pd
requirements, or upstream interaction.

## 2. Readiness and environment

Run only after M031–M037 close and documentation/support claims stabilize.

Required environment:

- local loopback bind;
- feature-enabled `emissary-cli` binary;
- temporary data directory;
- locally generated test TLS material or the service's bounded development
  certificate path;
- no public I2P exposure required for administrative-path validation;
- where full client/server traffic requires a functioning local SAM/router path,
  use the same Emissary process and bounded loopback endpoints.

If network formation/reseed prevents data-plane traffic, lifecycle readiness,
listener binding, task state, cancellation, identity persistence, and error
isolation remain mandatory evidence. Record any external-network blocker
precisely rather than replacing it with a fake success.

## 3. Required invariants

1. Validation runs against the real production composition and production
   adapters.
2. No fake control planes or fake backends are used for the principal scenario.
3. Loopback-only defaults and temporary paths are used.
4. Secrets/tokens/private destinations are not written to committed fixtures or
   logs.
5. The scenario is bounded in time, requests, tunnels, and output.
6. One failed request/tunnel cannot crash the router or I2PControl service.
7. Restart uses the same temporary state directory and verifies recovery.
8. Unsupported selectors/types return explicit errors and allocate no runtime
   resource.
9. No CI workflow, Docker privilege, root namespace, external router, or upstream
   dependency is required.
10. No production code change unless a direct defect is discovered and returned
    to a new corrective plan.

## 4. Scope and artifacts

Permitted additions:

- one local integration test, test-only harness, or script under existing test
  conventions;
- bounded fixtures containing no secrets;
- test documentation with exact commands;
- closure/evidence records.

Prefer a Rust integration test that launches the binary as a child process and
uses the existing HTTPS JSON-RPC stack. A small script is acceptable only if the
repository lacks a stable child-process test pattern.

Prohibited:

- `.github/workflows/**`;
- remote service dependencies;
- long soak tests;
- browser/UI automation;
- broad benchmark framework;
- production feature flags solely for tests;
- committed private keys/tokens/destinations;
- upstream actions.

## 5. Validation scenario

### Phase A — Build and start

1. Build `emissary-cli` with `--no-default-features --features i2pcontrol`.
2. Create a temporary base/config directory.
3. Configure loopback I2PControl with a unique ephemeral port and password.
4. Start Emissary and wait for bounded readiness.
5. Fail with captured sanitized diagnostics if the process exits or the service
   does not bind within the deadline.

### Phase B — Authentication and JSON-RPC

- successful `Authenticate`;
- wrong password/throttle behavior;
- protected request with `params.Token`;
- notification execution with no response;
- explicit-null and string/numeric request IDs;
- malformed/mixed mode errors.

### Phase C — AddressBook

- add a valid full destination to a bounded test book;
- list/lookup and RouterInfo list agreement;
- Base32/Base64 runtime resolution agreement where accessible;
- delete and stale-fallback absence;
- replace subscriptions through the M034 operational path or receive the exact
  documented unsupported status;
- reject unsafe `SetConfig` path keys.

### Phase D — RouterInfo and ClientServicesInfo

- request several available selectors and validate exact types;
- request one unavailable selector and confirm sanitized whole-request failure;
- query HTTPProxy/SOCKS/SAM/I2CP actual listener states;
- query I2PTunnel inventory before and after control-plane mutations;
- ensure only requested keys appear.

### Phase E — TunnelManager

Generic client:

- create stopped definition;
- get and inspect exact response;
- start and observe real runtime state/listener readiness;
- stop, restart, and delete;
- force one bind/config failure and then correct/restart without process or store
  reset.

Generic server:

- create/start;
- observe actual public destination metadata without private material;
- stop/restart and confirm identity stability;
- stopped rename identity preservation;
- delete and secret cleanup policy.

Unsupported type:

- create/get round-trip;
- start/restart explicit not-implemented status;
- stop safe/inactive;
- verify no listener/task/resource appears.

Startup-managed definition:

- observe inventory;
- verify mutation/lifecycle rejection.

### Phase F — Restart and recovery

1. Stop the process cleanly.
2. Restart with the same state directory.
3. Re-authenticate with a new token.
4. Verify durable AddressBook/tunnel definitions.
5. Verify eligible `StartOnLoad` behavior.
6. Verify server destination identity stability.
7. Confirm unsupported/startup boundaries remain.

### Phase G — Failure isolation and cleanup

- submit malformed/oversized/unknown requests within safe bounds;
- confirm service remains responsive;
- confirm one tunnel failure does not affect another;
- shut down process and verify no orphan child/listener/task;
- remove temporary secrets/state.

## 6. Evidence format

The test/harness output must record:

- binary commit/head and feature set;
- exact local command;
- phases and pass/fail status;
- sanitized request method/selector/action, never tokens/passwords/secrets;
- process exit and restart behavior;
- runtime listener/state assertions;
- known environmental blockers;
- elapsed wall time and enforced deadlines.

Do not commit generated runtime state as evidence. The closure record may contain
summarized results and deterministic fixture identifiers.

## 7. Failure and cancellation semantics

- Child process startup has a fixed deadline.
- Every HTTP request has a fixed deadline.
- Tunnel start/stop/restart has a fixed deadline.
- Test failure triggers child termination and bounded wait/kill fallback.
- Cleanup runs on assertion failure where the test framework permits.
- Port selection avoids fixed shared ports and checks actual bind.
- Network-dependent failure is distinguished from administrative-path failure.

## 8. Compatibility and migration

No production migration. The scenario validates existing persisted formats and
restart behavior only. Test artifacts are temporary and version-neutral.

## 9. Security review requirements

Review and test:

- loopback-only bind;
- no committed/generated secret leakage;
- TLS certificate/key temporary permissions;
- token/password redaction;
- server private destination redaction;
- bounded malformed request cases;
- no external endpoint or upstream interaction;
- deterministic cleanup.

## 10. Verification commands

At minimum:

```bash
cargo build -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

Use the repository's actual integration-test name if different. Run targeted
formatting and `git diff --check`.

No remote CI, platform matrix, coverage, fuzz, soak, container privilege, Java
I2P, i2pd, release, or packaging requirement.

## 11. Documentation and disposition

Create:

- `plans/closure/i2pcontrol-proposal-170/038-implementation-disposition.md`.

Update the support matrix with a distinction between:

- unit/fixture evidence;
- production-composition integration evidence;
- live child-process runtime evidence;
- any externally blocked data-plane traffic evidence.

## 12. Acceptance criteria

M038 may close only when:

- the real service starts and authenticates;
- all phases complete or a precise environment blocker is recorded;
- generic client/server lifecycle is exercised against production backends;
- unsupported and startup-owned behavior is truthful;
- restart/recovery and failure isolation pass;
- no secret is committed or logged;
- no production scope creep or CI expansion occurs;
- implementation disposition and frozen evidence head are committed;
- no upstream interaction occurred.

A handler-only/fake-backend test cannot substitute for this milestone.

## 13. Stop conditions

Stop and record `blocked` if:

- production composition cannot be launched in the available environment;
- safe temporary TLS/state handling is unavailable;
- the test requires privileged namespaces or external routers;
- a production defect is found that requires nontrivial correction; create a
  new corrective plan rather than patching it inside validation;
- external Proposal 170 authority changes materially;
- upstream action is requested without explicit authorization.