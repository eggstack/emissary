# M029 — In-Scope Proposal 170 Conformance Reclosure

Status: closed

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Source invalidation:

- `plans/closure/i2pcontrol-proposal-170/027-closure-invalidation.md`

Hard dependency:

- M028 closed with a frozen implementation/test head

Applicable governance and decision:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`

Pinned external authority:

- I2P Proposal 170, `I2PControl Expansion`
- status: `Open`
- created and last updated: `2026-05-20`
- existing I2PControl authentication and JSON-RPC contract

## 1. Objective

Perform a distinct internal review of the actual post-M028 repository head and
select the truthful final Proposal 170 disposition.

M029 is a closure/review plan. It must not become a second implementation pass.
It verifies that:

- the post-M027 planning regression is fully removed;
- the AddressBook control-state owner is isolated behind compile-time and
  runtime I2PControl enablement;
- retained M020–M027 wire, persistence, runtime-source, secret, and bounded
  observation behavior still passes;
- unsupported tunnel data planes and unavailable RouterInfo sources are
  represented truthfully;
- documentation and registry claims match actual production behavior; and
- all work remained internal to `eggstack/emissary`.

## 2. Activation conditions

M029 remains blocked until all of the following exist:

- `plans/closure/i2pcontrol-proposal-170/028-implementation-disposition.md`;
- `plans/closure/i2pcontrol-proposal-170/028-closure.md` with status `closed`;
- one frozen M028 implementation/test head;
- exact no-feature, runtime-disabled, enabled, and re-enable evidence;
- M028 registry status moved to closing/closed;
- no unresolved M028 high/medium correctness or security finding.

The M029 reviewer must be distinct from the final M028 implementation executor.
A different agent/run is sufficient; organizational upstream review is neither
required nor authorized.

## 3. Review baseline

The reviewer must record:

- the exact M028 frozen head;
- the exact current `master` head;
- whether they are identical;
- every commit after the M028 frozen head, if any;
- the final changed-file inventory from the pre-M028 baseline `03a384a`;
- the reviewer identity/run distinction from the M028 executor.

Any unreviewed production commit after the frozen M028 head blocks closure.
Documentation-only commits may be accepted only after explicit diff review.

## 4. External contract pin

Before reviewing repository claims, refetch read-only:

- Proposal 170;
- existing I2PControl API authentication/error documentation.

Record title, status, created date, last-updated date, and source location.

If Proposal 170 changed after `2026-05-20`, stop. Create a new contract-rebase
plan rather than silently adjudicating the change inside M029.

External source inspection is read-only. Do not open or update upstream issues,
pull requests, reviews, discussions, or patches.

## 5. Required review matrix

M029 must review each dimension independently.

| Dimension | Required conclusion |
|---|---|
| Wire | Exact existing-I2PControl and Proposal 170 request/response names, casing, presence rules, types, and error channels remain correct. |
| Source | Every field claimed available has a truthful bounded current owner. Unavailable fields remain explicit. |
| Runtime | Only real existing backends/services are called operational. Unsupported data planes never report running. |
| Persistence | AddressBook and TunnelManager mutations are durable, failure-atomic, restart-safe, and owner-coherent. |
| Feature isolation | Disabled I2PControl has no Proposal 170 control-state influence or persistence side effect. |
| Security | Authentication, path confinement, bounds, redaction, secret handling, and resource-free stubs remain intact. |
| Evidence | Literal fixtures, negative tests, production composition, restart, failure, and transition cases are independent of serializers under test. |
| Governance | Registry, roadmap, closure chronology, and internal-only boundary are accurate. |

A pass in one dimension cannot compensate for failure in another.

## 6. Required checks

### 6.1 Base I2PControl and JSON-RPC

Recheck:

- `Authenticate` accepts `API` and `Password` without mandatory username;
- `API` response is numeric;
- protected methods accept `params.Token`;
- header token remains compatibility-only and conflicts fail closed;
- error codes `-32001` through `-32006` remain distinct;
- notifications execute and suppress responses;
- explicit null IDs remain IDs;
- invalid IDs are rejected without coercion;
- existing base RouterInfo direct selectors still work.

### 6.2 AddressBook feature and owner boundaries

Recheck all four execution states:

1. no `i2pcontrol` feature;
2. feature compiled, runtime disabled;
3. feature and runtime enabled;
4. enabled, then disabled, then re-enabled across restarts.

Verify:

- disabled states do not read/write/consult control state;
- legacy lookup/download/persistence behavior remains functional;
- enabled state has one owner and immediate lookup visibility;
- current/backup recovery remains correct;
- disabling preserves but ignores control state;
- re-enabling restores it without duplicate authority;
- subscription/config shapes remain exact;
- no input selects arbitrary files;
- dependency feature ownership matches documentation.

### 6.3 TunnelManager

Recheck:

- seven lowercase canonical actions;
- twelve exact tunnel types;
- exact canonical `status`, `results`, `info`, and nested `rawConfig` shapes;
- strict option/range/enum validation;
- one-publication edit/rename;
- prior state after injected failure;
- secret omission from logs and responses;
- startup-owned name collision and mutation rejection;
- deterministic resource-free unsupported lifecycle behavior.

Do not require real backends for missing tunnel data planes.

### 6.4 ClientServicesInfo

Recheck:

- six direct selectors by parameter presence;
- startup/control-plane I2PTunnel inventory provenance;
- no local target host serialized as an I2P destination;
- HTTP/SOCKS state clears after task exit;
- SAM incomplete/recovery behavior remains bounded;
- I2CP reflects actual listener state;
- BOB remains exactly false.

### 6.5 RouterInfo

Recheck:

- exact 43 additions and literal JSON types;
- base/compatibility keys are separate;
- source count remains 16 available, 1 neutral, 26 unavailable;
- every available row has one bounded production owner;
- clock skew null is protocol-permitted neutral only;
- every unavailable row fails sanitized before partial assembly;
- no zero, false, empty collection, or adjacent metric is substituted;
- mixed/oversized/source-failure requests return no partial result.

M029 must not reopen M026 merely because unavailable fields remain. Their
implementation would require a separately authorized scope expansion.

## 7. Scope audit

Classify every file changed by M028 as one of:

- I2PControl production implementation;
- permitted AddressBook owner/composition seam;
- focused tests;
- directly affected documentation/planning.

Reject closure if the diff contains:

- router algorithm changes;
- transport, NetDB, peer-selection, streaming, LeaseSet, or cryptographic
  changes;
- new SAM behavior unrelated to a demonstrated defect;
- frontend changes;
- new dependencies or frameworks;
- missing tunnel data planes;
- CI/release/coverage/fuzz/soak machinery;
- broad formatting churn;
- upstream contribution preparation.

A small incidental compilation edit must be justified explicitly and shown to
be the narrowest option.

## 8. Failure, restart, cancellation, and contention review

The final closure record must address:

- enabled AddressBook current/backup corruption;
- disabled mode with stale/corrupt control-state files;
- enable/disable/re-enable transitions;
- download failure in both legacy and enabled modes;
- TunnelManager publication and rename failure;
- response loss after durable commit;
- startup tunnel collisions;
- proxy task exit and generation fencing;
- SAM incomplete and recovery states;
- RouterInfo source and response-bound failure;
- cancellation before and after publication;
- concurrent readers/mutators and lock-across-await review.

No new long-running test harness is required. Use deterministic existing/focused
tests and code inspection.

## 9. Security review

Search the final production and representative fixture/error paths for:

- passwords and tokens;
- proxy/outproxy/IRC credentials;
- private keys and key file paths;
- private destinations;
- arbitrary filesystem paths;
- raw control-state logging;
- unsupported backend resource allocation;
- disabled-mode control-state influence.

Verify authentication before protected work, restrictive permissions where
supported, atomic publication, temporary cleanup, path confinement, collection
bounds, sanitized errors, and no ordinary AddressBook mutation authority leaked
to unrelated consumers.

## 10. Required verification commands

Run focused M028 tests first, then:

```bash
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features address_book
cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings

cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol address_book
cargo test -p emissary-cli --no-default-features --features i2pcontrol tunnel_manager
cargo test -p emissary-cli --no-default-features --features i2pcontrol client_services
cargo test -p emissary-cli --no-default-features --features i2pcontrol router_info
cargo test -p emissary-cli --no-default-features --features i2pcontrol production_composition
cargo test -p emissary-cli --no-default-features --features i2pcontrol conformance_manifest
cargo test -p emissary-cli --no-default-features --features i2pcontrol m027_literal_fixtures
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings

cargo check -p emissary-core
cargo test -p emissary-core
cargo clippy -p emissary-core --all-targets -- -D warnings
```

Use touched-file formatting checks. Core commands revalidate retained SAM
behavior; they do not authorize new core work.

Remote CI, platform matrices, release validation, coverage, fuzzing, network
farms, soak tests, and generated evidence bundles are not required.

## 11. Documentation reconciliation

Before disposition, verify consistency across:

- `docs/i2pcontrol/README.md`;
- `docs/i2pcontrol/address-book.md`;
- `docs/i2pcontrol/proposal-170-support.md`;
- `docs/i2pcontrol/proposal-170-conformance.md`;
- `docs/i2pcontrol/router-info-source-map.md`;
- `plans/registry.md`;
- subsystem roadmap;
- implementation README;
- M027 invalidation;
- M028 disposition/closure;
- M029 closure.

Required chronology:

- M019 is superseded historical evidence;
- M020–M027 are retained corrective evidence;
- M027 final disposition was invalidated after the post-M027 findings;
- M028 is the feature-isolation/status implementation correction;
- M029 is the controlling final-head review.

Do not delete historical records solely to make the chronology simpler.

## 12. Final disposition rules

M029 may choose only one:

### Partial Proposal 170 support

Use when:

- every implemented and claimed dimension passes;
- no unresolved high/medium correctness or security defect remains;
- one or more pinned sources remain truthfully unavailable; or
- one or more declared tunnel types remain explicit unsupported runtimes.

This is the expected disposition under the current authorized scope.

### Closed internally against pinned revision

Use only if every Proposal 170 source and runtime dimension claimed by the
proposal is actually available and evidenced. Parser coverage, stored
configuration, explicit errors, or stubs are insufficient.

M029 does not authorize the implementation work required to reach this status.

### Corrective pass required

Use for any unresolved high/medium defect, documentation contradiction, feature
leak, fabricated value, secret/path issue, atomicity problem, or unreviewed
production diff.

### Blocked

Use when the proposal revision changed, required evidence cannot be obtained, or
M028 did not produce a stable reviewable head.

No disposition implies upstream review, acceptance, certification, adoption, or
merge.

## 13. Acceptance criteria

M029 closes only when:

1. the proposal pin is independently verified;
2. reviewer independence is recorded;
3. current master matches the reviewed frozen head or all later diffs are
   explicitly accepted;
4. M028 feature-isolation requirements pass in all four execution states;
5. M020–M027 retained method-level requirements pass;
6. the 16/1/26 matrix is unchanged unless a separately authorized source plan
   landed before activation;
7. unsupported tunnel data planes remain explicit and resource-free;
8. no high/medium correctness, security, compatibility, or claim defect remains;
9. documentation and registry chronology is consistent;
10. all required local commands pass or an unrelated baseline failure is
    precisely isolated without weakening changed-path evidence;
11. no prohibited scope expansion occurred;
12. no upstream write, review request, submission, adoption request, or merge
    solicitation occurred;
13. a complete final closure record is committed after the reviewed head is
    frozen.

## 14. Stop conditions

Stop and reject closure if:

- the external proposal revision changed;
- M028 lacks disabled/runtime-disabled evidence;
- control state influences disabled lookup or persistence;
- two AddressBook authorities exist;
- a schema migration is hidden inside M028;
- any canonical fixture fails;
- a secret or private path appears in unintended output;
- an unsupported backend opens resources or reports running;
- an unavailable RouterInfo field is fabricated;
- a post-freeze production commit is unreviewed;
- scope expanded into core/router/data-plane/CI work;
- M019 is still represented as controlling closure;
- upstream interaction occurred.

## 15. Required closure output

Create:

- `plans/closure/i2pcontrol-proposal-170/029-closure.md`

It must contain:

- exact source pin and read-only evidence;
- reviewer independence statement;
- reviewed/frozen heads;
- final changed-file classification;
- requirement-to-evidence matrix;
- exact commands and outcomes;
- feature-isolation transition evidence;
- failure/restart/cancellation/contention review;
- compatibility/migration review;
- security review;
- RouterInfo 16/1/26 and tunnel-runtime disposition;
- unresolved findings with severity;
- final status;
- internal-only/no-upstream attestation.

After acceptance, update registry, roadmap, implementation README, and support
documentation to identify M029 as the controlling internal closure. Preserve
M019, M019A, M027, and their invalidations as historical records.
