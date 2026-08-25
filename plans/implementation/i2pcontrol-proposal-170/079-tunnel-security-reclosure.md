# M079 — Proposal 170 Tunnel Security Reclosure

Status: closed — historical older-lineage closure; M085 is current-head authority

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`.

Source runtime reclosure:

- `plans/closure/i2pcontrol-proposal-170/072-closure.md`.

Original corrective-series baseline: `04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`.

Post-M076 corrective baseline: `1618de172e7a78a193fc1bb117af269f31174030`.

## 1. Objective

Independently reclose the Proposal 170 tunnel runtime/security workstream after the complete corrective sequence. M079 is evidence/reconciliation work, not a feature milestone.

It must verify the actual final repository head and must not trust historical M073-M076 closure assertions where later regressions or independent findings superseded them. Any material defect found here receives a new narrowly scoped corrective plan rather than being fixed opportunistically inside M079.

## 2. Required predecessor state

M079 may begin only after independent closure of:

- M080 — admission transactionality/cardinality/canonical peer identity;
- M081 — generic server `leaseSetEncType` apply-or-reject truthfulness;
- M082 — HTTP Destination/Expect/POST identity corrective;
- M077 — IRC lifetime/exhaustion hardening;
- M078 — Streamr local-boundary hardening.

Historical M074-M076 evidence remains relevant for retained behavior, but those milestones are not sufficient prerequisites without their corrective successors.

## 3. Scope

Review all twelve production tunnel types:

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

Do not reopen unrelated RouterInfo 37/1/5, AddressBook limitations, base I2PControl methods, or absent source owners except where final documentation must remain truthful.

## 4. Independent threat-model review

### 4.1 Resource exhaustion / fairness

Prove:

- accepted server state is bounded globally and per peer;
- one authenticated peer cannot consume the global pool;
- peer/aggregate minute/hour/day controls work across boundaries;
- every denial path is side-effect free except bounded expiry reclamation;
- aggregate-rejected fresh identities do not create peer records;
- peer-state capacity is coherent with enabled retained windows and configured aggregate rates or unsafe configurations reject before allocation;
- every auxiliary expiry/index structure is hard bounded and cannot accumulate stale entries independently of the primary map;
- active/throttled state is not evicted merely to admit an attacker-controlled identity;
- task/permit/state release occurs on all error/panic/cancel/abort paths;
- generic server participates in the same corrected admission boundary;
- HTTP POST limiter remains bounded/churn-safe and its auxiliary expiry state is bounded;
- IRC registered-idle connections expire;
- Streamr subscriber/fanout state remains finite.

### 4.2 Timing/correlation

Required conclusion/evidence:

- no single peer can reliably toggle global capacity by monopolizing all permits;
- aggregate/table denial cannot be made persistent through state poisoning;
- local target failure details are not exposed;
- no artificial jitter/sleep defense exists;
- unavoidable application/I2P latency is documented as residual risk rather than falsely eliminated;
- overload/idle policy does not create an unbounded attacker-controlled wait;
- unsupported HTTP expectation semantics cannot create a deterministic client/backend wait until body timeout.

Do not require constant-time networking or public deanonymization experiments.

### 4.3 Trusted peer identity

Prove:

- accepted SAM/Yosemite peer identity is structurally valid as an I2P Destination before security accounting/protocol metadata use;
- canonical fixed-size cryptographic Destination ID/hash keys admission and HTTP POST accounting;
- valid current Destination forms using larger supported key-certificate/signature types are accepted;
- malformed/invalid Destination text is rejected before handler/local target work;
- any retained full Destination text has a documented bound derived from current supported representation, not a legacy magic size;
- full peer Destination text is absent from high-cardinality diagnostics.

### 4.4 HTTP identity/fingerprint/framing boundary

Prove:

- spoofed request-side I2P/proxy identity headers cannot reach the loopback backend;
- trusted injected B64/B32 identity corresponds to the authenticated parsed Destination;
- `Date`, `Server`, and adopted provider/cache/trace headers cannot reach I2P;
- request/response framing remains unambiguous after filtering;
- any `Expect` request fails before local target allocation unless a separately accepted plan explicitly implemented full informational-response support;
- POST limiter cannot be bypassed by active-entry churn and uses canonical peer identity;
- `httpbidirserver` uses the exact same inbound server filter/admission behavior.

### 4.5 Generic server option truthfulness

Prove:

- generic control-plane `server` still uses accepted streams and does not issue `STREAM FORWARD`;
- payload remains byte-transparent after admission;
- `leaseSetEncType` is demonstrably applied to accepted-stream session setup or is rejected before allocation;
- no other recognized runtime-relevant field is accepted but ignored;
- startup-managed server behavior remains independently owned and unchanged unless a separately authorized plan says otherwise.

### 4.6 IRC boundary

Prove:

- registration filtering precedes local connect;
- hostname identity comes only from trusted accepted I2P Destination;
- wrong protocol/malformed registration remains fail-closed;
- 10-minute inactivity is activity-resetting, not total-lifetime;
- target connect is bounded;
- DCC/WEBIRC unsupported paths remain explicit/non-bypassable;
- server/client filter roles remain distinct.

### 4.7 Streamr boundary

Prove:

- local UDP producer and client target are loopback-only;
- remote packets never choose local target;
- subscriber ceiling is reference-aligned and finite;
- expiry/refresh/control/payload bounds remain exact;
- trusted Destination validation is compatible with the corrected common identity boundary;
- no amplification task queue or subscriber eviction path exists.

## 5. Option-truthfulness reconciliation

Rebuild/update the integrated option-capability matrix for all twelve tunnel types.

For every runtime-relevant field classify exactly one:

- implemented and demonstrably applied;
- irrelevant/invalid for the tunnel type and rejected;
- recognized but unsupported and rejected before allocation.

No persist-and-ignore row may remain.

Specifically re-audit:

- `MaxConcurrentConns`;
- `ClientPerMinute/Hour/Day`;
- `TotalInPerMinute/Hour/Day`;
- `PostLimit`/`PostLimitTime`;
- access-list fields;
- `PerClientPeriod`, `TotalPeriod`, `TotalBanTime`;
- `FilterFilePath`;
- `UniqueLocalAddressPerClient`;
- `MultiHoming`;
- `leaseSetEncType` and other declared LeaseSet/session shaping fields;
- generic client/server raw option allowlists.

If a field remains underspecified by the pinned Proposal/reference, final support docs must say it is rejected.

## 6. Containment review

Compare final head against `1618de1` and the earlier planning baseline/accepted M061-M063 authorities.

Required:

- expected production changes under `emissary-cli/src/i2pcontrol/**` only;
- no new `emissary-core/**` production path;
- no unrelated startup proxy/tunnel ownership refactor;
- no unnecessary default-enabled dependency/feature widening;
- any I2PControl-only dependency remains optional/feature-owned;
- re-evaluate Tokio `test-util` placement and record the final production/dev dependency state;
- default/feature-disabled builds remain unaffected;
- no hosted CI/release/fuzz/soak/public deanonymization infrastructure.

A containment deviation is medium severity unless proven purely documentary/test-local.

## 7. Lifecycle/restart review

For each real backend verify:

- start validates before allocation;
- running is reported only after real runtime readiness;
- stop is idempotent;
- restart is complete stop then new generation;
- old generation cannot mutate new state;
- child tasks are bounded/drained/aborted within declared timeout;
- server public destination remains stable when persistent identity remains;
- ephemeral admission/rate/POST/subscriber state is cleared on restart;
- one failed tunnel does not fail unrelated definitions/StartOnLoad reconciliation;
- old admission leases cannot mutate a replacement generation.

## 8. Required adversarial integration evidence

Reuse predecessor tests and add only missing integrated cases. Required evidence includes:

- exhaust aggregate rate, then submit more fresh valid Destinations than the admission state ceiling; peer/expiry sizes do not grow on denied attempts;
- repeated same-peer acquire/drop does not grow stale expiry metadata without bound;
- attacker peer fills its per-peer concurrent ceiling while a second peer still succeeds;
- unsafe configured rate/capacity combination fails before SAM allocation;
- generic server with `leaseSetEncType` proves apply-or-reject at runtime;
- generic server admitted bytes are raw/unchanged and denied peers never connect target;
- valid large current I2P Destination reaches HTTP backend identity headers correctly;
- malformed Destination never reaches local backend;
- `Expect: 100-continue`/other expectations receive bounded rejection with no local accept;
- HTTP backend emits `Date`, provider/cache/trace headers and none reach peer;
- request carries spoofed proxy/I2P identity names and none reach backend;
- POST churn cannot reset an active peer limit and POST accounting uses canonical Destination ID;
- IRC registered idle stream expires while active IRC stream persists;
- Streamr non-loopback config rejects before allocation;
- Streamr 11th subscriber is rejected while existing 10 remain;
- stop/restart during active admitted connections leaves no stale permits/tasks/accounting.

Use local fake SAM/TCP/UDP fixtures only.

## 9. Verification commands

At minimum record exact outcomes for:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

Also identify/run focused admission, generic-server, HTTP, IRC, and Streamr suites. Use repository-accepted nightly rustfmt only for touched files where needed; record inherited unrelated toolchain/format limitations instead of expanding scope.

## 10. Documentation reconciliation

Update as needed:

- `docs/i2pcontrol/proposal-170-support.md`;
- `docs/i2pcontrol/tunnel-manager.md`;
- `docs/i2pcontrol/tunnel-backends.md`;
- runtime/security roadmaps;
- implementation README;
- `plans/registry.md`;
- final option capability matrix/closure evidence.

Documentation must distinguish functional type availability from security closure, supported from rejected options, loopback policies, and unrelated remaining Proposal 170 source limitations.

## 11. Closure record requirements

Create `plans/closure/i2pcontrol-proposal-170/079-closure.md` only after evidence is complete.

It MUST include:

- exact implementation commit ranges for M080-M082/M077-M078 and retained historical M074-M076 evidence;
- requirement-to-evidence matrix;
- exact commands/outcomes;
- threat-model/anonymity/resource review;
- transactional denial/cardinality/expiry evidence;
- trusted Destination identity evidence;
- generic option-truthfulness evidence;
- HTTP Expect/fingerprint/framing/POST evidence;
- IRC idle evidence;
- Streamr local-boundary evidence;
- lifecycle/contention review;
- option-capability matrix;
- containment/dependency diff review;
- unresolved findings with severity;
- external read-only research attestation;
- explicit statement that no upstream issue/PR/review/submission/write occurred.

## 12. Acceptance criteria

M079 may close only when:

1. M080, M081, M082, M077, and M078 are independently closed;
2. admission denial is transactional and cannot poison peer state;
3. peer/expiry/accounting state is hard bounded and capacity is coherent with retained semantics;
4. security accounting uses canonical I2P Destination identity;
5. one peer cannot monopolize accepted-server capacity;
6. generic `server` remains accepted-stream and all runtime-relevant options are applied or rejected;
7. valid current I2P Destination forms are accepted and malformed identities fail before local work;
8. unsupported HTTP expectations cannot pin body timeout/local backend wait;
9. Java-parity/stronger HTTP fingerprint stripping remains verified;
10. HTTP limiter state is churn-safe/bounded;
11. IRC registered-idle occupancy is finite/activity-resetting;
12. Streamr local UDP exposure is loopback-only and fanout bounded;
13. M061/M062/M063 containment remains intact;
14. no production core/startup-service scope expansion occurred;
15. support documentation is truthful;
16. no high/medium security, anonymity, correctness, lifecycle, option-truthfulness, or containment finding remains;
17. no upstream interaction occurred.

## 13. Corrective disposition rule

Any high/medium finding means `corrective pass required`, not conditional closure. Create one narrow successor plan for the exact defect and keep M079/security roadmap open.

Tiny documentation/test-name corrections may be made during reclosure; material runtime/security changes must not originate inside M079.
