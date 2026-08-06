# M044 — Corrective Final-Head Reclosure

Status: closed; partial Proposal 170 support

Hard dependency:

- M043 closed

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Corrective authority:

- `plans/closure/i2pcontrol-proposal-170/039-closure-invalidation.md`

Applicable governance:

- `plans/003-planning-process.md`
- retained M020–M039 records, subject to the M039 invalidation

## 1. Bounded objective

Independently review the final M040–M043 corrective head and select the truthful
Proposal 170 subsystem disposition.

M044 is review and documentation only. It must not patch production or test code.
Any material defect found during M044 requires a new corrective implementation
plan and leaves the subsystem at `corrective pass required`.

The expected status is `partial Proposal 170 support`, not full completion,
because ten tunnel families and 26 RouterInfo sources remain intentionally
unsupported or unavailable under the controlling roadmap.

## 2. Required review baseline

M044 must identify and freeze:

- M040 implementation/test head and closure;
- M041 implementation/test head and closure;
- M042 implementation/test head and closure;
- M043 evidence head and closure;
- the exact final repository head reviewed;
- the unchanged pinned Proposal 170 revision.

Review must be performed against the final head, not only component commits or
agent summaries.

## 3. Required review dimensions

### 3.1 Startup server preservation

Confirm that the original startup `ServerTunnelManager` retains its cancellation
sender, reaches SAM session creation and `STREAM FORWARD`, publishes destination
metadata, and remains externally owned/read-only.

Confirm that no administrative handle or adoption path was added.

### 3.2 Control-plane tunnel lifecycle

Reconfirm:

- real generic client/server backends;
- per-name supervision and generation fencing;
- fixed server-secret ownership;
- start/stop/restart/delete/edit/rename/StartOnLoad behavior;
- startup/control-plane ownership separation;
- explicit resource-free unsupported status for the other ten families.

### 3.3 Authentication

Confirm:

- reviewed constant-time password comparison;
- throttle key normalized to source IP;
- atomic failure reservation before sleep;
- bounded table/window/delay;
- lock released before await;
- success clears source-IP state;
- exact authentication errors, API versions, and token behavior unchanged.

### 3.4 AddressBook

Confirm:

- one enabled-mode full-destination owner;
- feature-disabled/runtime-disabled isolation;
- live and durable subscription replacement;
- one explicit mutation linearization point;
- no failure response after durable commit;
- refresh is bounded follow-up work;
- non-empty `SetConfig` remains explicit unsupported;
- no arbitrary path or second authority.

### 3.5 Compatibility and wire

Reconfirm exact Proposal 170 names, casing, types, presence semantics, action and
tunnel inventories, direct/base compatibility modes, and explicit unsupported
base methods.

### 3.6 RouterInfo and ClientServicesInfo

Reconfirm:

- 16 available, 1 neutral, and 26 unavailable RouterInfo additions;
- no fabricated source values;
- actual listener/proxy/tunnel owners;
- bounded passive SAM observation;
- no frontend ownership.

### 3.7 Persistence and security

Reconfirm path confinement, permissions, atomic publication, backup recovery,
directory-sync qualifications, redaction, request/connection/concurrency bounds,
and absence of arbitrary request-selected secret paths.

### 3.8 Containment

Classify every production change from `563e093` to final head. The corrective
sequence should add only narrow changes in the startup server adapter,
I2PControl authentication/server paths, and AddressBook command path.

No new core behavior, tunnel family, RouterInfo source, frontend, workflow, or
release infrastructure is permitted.

### 3.9 Evidence quality

Review exact M043 commands and outcomes. Distinguish:

- repository-recorded local evidence;
- any attached CI evidence, if present;
- qualified environmental limitations;
- untested claims.

Do not convert a no-peer or no-downloader limitation into fabricated positive
evidence.

## 4. Requirement-to-evidence matrix

The M044 closure must include a table covering at least:

- authentication and token behavior;
- normalized/atomic failed-auth throttle;
- direct/base compatibility;
- RouterInfo contract/source matrix;
- AddressBook entries/subscriptions/configuration;
- TunnelManager wire and lifecycle;
- startup server preservation;
- generic client/server backends;
- unsupported tunnel families;
- startup ownership;
- StartOnLoad and failure recovery;
- ClientServicesInfo and SAM observation;
- persistence/recovery/durability;
- feature isolation;
- secret handling and resource bounds;
- containment;
- live/focused runtime validation;
- internal-only/no-upstream compliance.

Each row must cite exact files/tests/commands and choose pass, qualified pass,
failed, or unavailable. Broad assertions without source evidence are not
sufficient.

## 5. Allowed status outcomes

### `partial Proposal 170 support`

Allowed only if every implemented and claimed dimension is exact, operational,
bounded, and evidenced, while the ten unsupported tunnel families and 26
unavailable RouterInfo sources remain explicit.

### `corrective pass required`

Required if any high/medium correctness, security, ownership, compatibility,
containment, persistence, or evidence defect remains.

### `blocked`

Required if the final head, pinned specification, or required evidence cannot be
reliably reviewed.

`closed internally against pinned revision` or full completion is not available
under this roadmap because declared capabilities remain unavailable.

## 6. Verification

M044 should independently rerun a representative subset rather than merely copy
M043 output. At minimum:

```bash
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m033_tunnel_lifecycle
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo test -p emissary-core sam
git diff --check
```

Also rerun the exact focused M040–M042 regressions by their final target names.

No remote CI, release, coverage, fuzz, or soak expansion is required.

## 7. Documentation and registry disposition

M044 must update, according to its finding:

- `plans/registry.md`;
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- directly affected `docs/i2pcontrol/**` support/security/tunnel/address-book
  summaries;
- a new M044 closure record.

`039-closure.md` and `039-closure-invalidation.md` remain historical records and
must not be deleted or rewritten.

No successor plan becomes ready unless M044 finds another material defect and a
new bounded corrective plan is explicitly registered.

## 8. Acceptance criteria

M044 may close only when:

- the exact final head is identified and cleanly reviewable;
- the pinned Proposal 170 revision is unchanged or a rebase blocker is recorded;
- M040–M043 closures and focused regressions are independently checked;
- the startup server, auth throttle, and AddressBook commit-boundary defects are
  demonstrably corrected;
- retained M031–M039 functionality has not regressed;
- changed paths remain within the roadmap boundary;
- all high/medium findings are resolved or the status remains corrective;
- unsupported/unavailable capabilities remain explicit;
- no production/test patch is included in M044;
- internal-only/no-upstream attestation is present.

## 9. Stop conditions

Stop and select `corrective pass required` rather than patching if:

- any exact-path regression fails;
- startup server tasks still self-cancel;
- port churn or concurrent auth bypass remains;
- subscription mutation can still return failure after commit;
- new core behavior or scope expansion is found;
- documentation materially overstates evidence;
- the proposal revision changed;
- upstream interaction occurred.

## 10. Closure evidence required

The M044 closure must contain:

- final reviewed head;
- dependency/closure SHAs;
- requirement-to-evidence matrix;
- exact independent verification commands/outcomes;
- invariant, failure/recovery, contention, compatibility, migration, security,
  and containment review;
- changed-path classification;
- environmental qualifications;
- unresolved findings with severity;
- final truthful status;
- explicit statement that ten tunnel families and 26 RouterInfo sources remain
  outside implemented support;
- internal-only/no-upstream attestation.
