# M079 — Proposal 170 Tunnel Security Reclosure

Status: closed — implementation and independent closure accepted

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`.

Source runtime reclosure:

- `plans/closure/i2pcontrol-proposal-170/072-closure.md`.

Planning baseline for the corrective series: `04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`.

## 1. Objective

Independently reclose the Proposal 170 tunnel runtime after the server/anonymity hardening sequence. M079 is evidence/reconciliation work, not a feature milestone.

It must verify the actual final repository head, not trust M074-M078 plan assertions. Any material defect found here receives a new narrowly scoped corrective plan instead of being hidden inside closure.

## 2. Scope

Review all twelve production tunnel types, with deep emphasis on the newly introduced specialized families and the generic server path:

Client-side:

- `client`;
- `httpclient`;
- `ircclient`;
- `socks`;
- `socksirc`;
- `connectclient`;
- `streamrclient`.

Server-side:

- `server`;
- `httpserver`;
- `httpbidirserver`;
- `ircserver`;
- `streamrserver`.

Do not reopen unrelated RouterInfo 37/1/5, AddressBook limitations, base I2PControl methods, or deferred source owners except where documentation must remain truthful.

## 3. Required independent threat-model review

### 3.1 Resource exhaustion / fairness

Prove:

- accepted server state is bounded globally and per peer;
- one authenticated peer cannot consume the entire global pool;
- peer/aggregate connection windows work across minute/hour/day boundaries;
- limiter table full behavior fails closed without active-state eviction;
- task/permit/state release occurs on all error/panic/cancel paths;
- generic server participates in the same admission boundary;
- IRC registered-idle connections expire;
- Streamr subscriber/fanout state remains bounded.

### 3.2 Timing/correlation

Review externally controllable timing observables with a realistic anonymity model.

Required conclusion/evidence:

- no single peer can reliably toggle the global capacity state;
- local target failure details are not exposed;
- no artificial jitter/sleep defense was introduced;
- unavoidable application response latency is documented as residual risk rather than falsely claimed eliminated;
- overload/idle policy does not create an unbounded attacker-controlled server wait.

Do not require constant-time networking or public deanonymization experiments.

### 3.3 HTTP identity/fingerprint boundary

Prove:

- spoofed request-side I2P/proxy identity headers cannot reach the loopback backend;
- trusted peer identity injection is bounded;
- `Date`, `Server`, and the adopted fingerprint/provider/cache/trace headers cannot reach I2P;
- HTTP framing remains unambiguous after filtering;
- POST limiter cannot be bypassed by active-entry churn;
- `httpbidirserver` uses the exact same inbound server filter/admission behavior.

### 3.4 IRC boundary

Prove:

- registration filtering still precedes local connect;
- hostname identity comes only from the accepted I2P destination;
- wrong protocol/malformed registration remains fail-closed;
- 10-minute inactivity is activity-resetting, not total-lifetime;
- DCC/WEBIRC unsupported paths remain explicit and non-bypassable;
- server/client filter roles remain distinct.

### 3.5 Streamr boundary

Prove:

- local UDP producer and client target are loopback-only;
- remote packets never choose local target;
- subscriber ceiling is reference-aligned and finite;
- expiry/refresh/control/payload bounds remain exact;
- no unexpected amplification task queue or subscriber eviction behavior exists.

## 4. Option-truthfulness reconciliation

Rebuild or update the integrated option-capability matrix for all twelve tunnel types.

For each Proposal 170 runtime-relevant field classify exactly one of:

- implemented and demonstrably applied;
- irrelevant/invalid for the tunnel type and rejected;
- recognized but unimplemented and rejected before allocation.

No persist-and-ignore row may remain.

Specifically re-audit server controls:

- `MaxConcurrentConns`;
- `ClientPerMinute/Hour/Day`;
- `TotalInPerMinute/Hour/Day`;
- `PostLimit`/`PostLimitTime`;
- access-list fields;
- `PerClientPeriod`, `TotalPeriod`, `TotalBanTime`;
- `FilterFilePath`;
- `UniqueLocalAddressPerClient`;
- `MultiHoming`;
- LeaseSet/session shaping fields.

If a field remains underspecified by the pinned Proposal/reference, the final support docs must say it is rejected; do not convert uncertainty to implied support.

## 5. Containment review

Compare the final corrective-series head against planning baseline `04e0c2e` and the accepted M061/M062/M063 authorities.

Required:

- expected production changes under `emissary-cli/src/i2pcontrol/**` only;
- no new `emissary-core/**` production path;
- no unrelated startup proxy/tunnel ownership refactor;
- no new default-enabled dependency;
- any I2PControl-only dependency remains optional and feature-owned;
- default/feature-disabled builds remain unaffected;
- no new hosted CI/release/fuzz/soak infrastructure.

A containment deviation is medium severity unless proven purely documentary/test-local.

## 6. Lifecycle/restart review

For each real backend verify:

- start validates before allocation;
- running is reported only after real runtime readiness;
- stop is idempotent;
- restart is complete stop then new generation;
- old generation cannot mutate new state;
- child tasks are bounded/drained/aborted within declared timeout;
- server public destination remains stable when persistent identity remains;
- ephemeral rate/subscriber state is not accidentally persisted across restart;
- one failed tunnel does not fail unrelated definitions/StartOnLoad reconciliation.

## 7. Required adversarial integration tests

M079 should reuse tests added by M074-M078 and add only missing integrated cases. Required evidence includes:

- attacker peer fills its per-peer concurrent ceiling while a second peer still succeeds;
- aggregate ceiling eventually denies distinct peers without growing memory;
- limiter-table saturation cannot erase attacker state;
- HTTP backend emits `Date`, provider/cache/trace headers and none reach peer;
- request carries multiple spoofed proxy identity names and none reach backend;
- POST churn across many peer identities cannot reset an active peer's limit;
- IRC registered idle stream expires while active IRC stream persists;
- generic server admitted bytes are raw/unchanged and denied peers never connect target;
- Streamr non-loopback config rejects before allocation;
- Streamr 11th subscriber rejected while existing 10 remain;
- stop/restart during active admitted connections leaves no stale permits/tasks.

Use local fake SAM/TCP/UDP fixtures. Do not perform deanonymization experiments against the public I2P network.

## 8. Verification commands

At minimum record exact outcomes for:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

Also run/identify:

- M061 source containment suite;
- M062/M063 dependency containment suite;
- focused accepted-server admission tests;
- focused generic server fake-SAM tests;
- focused HTTP filter/POST limiter tests;
- focused IRC paused-time tests;
- focused Streamr paused-time/address tests.

Use repository-accepted nightly rustfmt only for touched files if needed. Existing unrelated repository-wide formatting/toolchain limitations must be recorded rather than expanded into this scope.

## 9. Documentation reconciliation

Update, as needed:

- `docs/i2pcontrol/proposal-170-support.md`;
- `docs/i2pcontrol/tunnel-manager.md`;
- `docs/i2pcontrol/tunnel-backends.md`;
- the tunnel runtime/security roadmaps;
- `plans/registry.md`;
- final option capability matrix/closure evidence.

Documentation must distinguish:

- functional type availability;
- server anonymity/resource protections;
- supported/rejected server options;
- loopback-only Streamr policy;
- remaining Proposal 170 RouterInfo/AddressBook limitations;
- lack of public-network certification as an environmental evidence limitation, not a hidden implementation claim.

## 10. Closure record requirements

Create `plans/closure/i2pcontrol-proposal-170/079-closure.md` only after implementation evidence is complete.

The closure record MUST include:

- exact implementation commit range for M073-M078;
- requirement-to-evidence matrix;
- all commands and outcomes;
- threat-model/anonymity review;
- limiter/fairness/idle/fingerprint evidence;
- lifecycle/contention review;
- option-capability matrix disposition;
- containment/dependency diff review;
- unresolved findings with severity;
- external read-only research attestation;
- explicit statement that no upstream issue/PR/review/submission/write occurred.

## 11. Acceptance criteria

M079 may close only when:

1. M073-M078 are independently closed;
2. one peer cannot monopolize accepted-server global capacity;
3. generic `server` uses peer-aware accepted streams or a separately accepted equivalent with the same protections;
4. HTTP Java-parity fingerprint stripping includes `Date`, and the stronger adopted anonymity set is verified;
5. HTTP limiter state is churn-safe and bounded;
6. IRC registered-idle occupancy is finite and activity-resetting;
7. Streamr local UDP exposure is loopback-only and fanout remains bounded;
8. every runtime-relevant option is applied or rejected before allocation;
9. M061/M062/M063 containment remains intact;
10. no production core/startup-service scope expansion occurred;
11. no high/medium security, anonymity, correctness, lifecycle, or containment finding remains;
12. support documentation is truthful;
13. no upstream interaction occurred.

## 12. Corrective disposition rule

Any high/medium finding means `corrective pass required`, not conditional closure. Create one narrow successor plan for the exact defect and keep M079/the security roadmap open.

Tiny documentation/test-name corrections may be made during reclosure; material runtime/security changes must not originate inside M079.
