# M052 — RouterInfo 26-Source Integration, Containment Review, and Reclosure

Status: closed

Planning baseline: `b759038`

Closure record: `plans/closure/i2pcontrol-proposal-170/052-closure.md`

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Hard dependencies: M045–M051 closed or explicitly blocked with accepted semantic disposition

Milestone class: closure/evidence gate

## 1. Objective

Independently validate the complete line of work for the 26 previously unavailable Proposal 170 RouterInfo additions. M052 is validation and reclosure only; it must not patch production code.

The target is RouterInfo-dimension completion against the pinned Proposal 170 revision while preserving the broader repository status as `partial Proposal 170 support` if unrelated unsupported tunnel families, `SetConfig`, or other accepted partial dimensions remain.

## 2. Invariants

- No production changes in M052. A material defect creates a new corrective implementation plan.
- Every field reported available has a named authoritative source and exact type/shape fixture.
- No literal placeholder zero/false/empty/null is accepted as source evidence unless the contract explicitly permits that neutral/current value and the owner was queried successfully.
- Core changes from M046–M050 are neutral read-only/passive observation only, contain no Proposal 170/wire terminology, and introduce no mutable control authority.
- Default/no-feature behavior is unchanged.
- No upstream interaction is authorized.

## 3. Review matrix

Review all 26 original unavailable rows, grouped as:

- known peer directory: 3;
- active peer inventory/limits: 4;
- active peer stats: 1;
- tunnel counts/details: 7;
- rolling metrics/queues: 4;
- v4/v6 network state: 5;
- news/banned peers: 2.

Also rerun the retained 16 available + 1 protocol-neutral rows to detect composition regressions. Confirm the exact 43-row `PROPOSAL_170_CONTRACT` inventory and source-map documentation agree.

## 4. Containment audit

Compare the final head against M044 and classify every production change as:

1. I2PControl policy/adapter code;
2. CLI composition-only wiring;
3. neutral core inspection DTO/handle;
4. minimal passive observation at an authoritative core owner.

Any production change outside the authorized M045–M051 path budget is a closure blocker unless separately planned and closed. Specifically inspect crypto, I2NP, routing, tunnel selection/build algorithms, transport state machines, NetDB protocol behavior, proxy/UI, AddressBook, and workflows for unauthorized changes.

## 5. Operational evidence

Exercise the real feature-enabled child process with TLS/authentication and request each newly available field individually. Exercise representative multi-selector requests from independent source groups. Where deterministic live state is hard to create, pair the process-level smoke path with bounded production-composition fixtures that use the exact production adapter.

No public-network certification is required; tests may use loopback/fake peer sources where protocol formation is not relevant, but they must exercise the production composition boundary.

## 6. Verification commands

At minimum:

```bash
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features
cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo check -p emissary-core
cargo test -p emissary-core --no-fail-fast
cargo clippy -p emissary-core --all-targets -- -D warnings
git diff --check
```

Use targeted formatting due the documented formatter baseline mismatch. No new CI/release/fuzz/soak infrastructure.

## 7. Acceptance criteria

M052 may close RouterInfo source completion only if every one of the 26 target fields is either:

- operational from a truthful bounded authoritative source with exact wire evidence; or
- explicitly retained blocked by an accepted M051-style semantic limitation, in which case RouterInfo completion is not claimed.

If all 26 become operational, update source counts from 16 available / 1 neutral / 26 unavailable to 42 available / 1 protocol-permitted neutral / 0 unavailable. Confirm no source relies on a placeholder or data-plane mutation.

The overall Proposal 170 subsystem must remain `partial Proposal 170 support` unless a separate authorized roadmap closes all other unsupported dimensions. M052 does not authorize that broader claim.

## 8. Stop/disposition rules

- Any high/medium correctness, security, containment, or source-truthfulness defect: `corrective pass required`.
- Required evidence unavailable: `blocked`.
- One or more of the 26 remains semantically unavailable: `RouterInfo source completion incomplete`; retain truthful partial matrix.
- All 26 operational with clean containment: `RouterInfo source completion closed internally against pinned revision`; broader Proposal 170 status remains partial as applicable.

## 9. Closure evidence

Create an independent closure record with exact final head, requirement-to-evidence matrix for all 26 target rows, commands/outcomes, changed-path classification, failure/recovery/contention review, source-count reconciliation, documentation review, and explicit read-only-external/internal-only attestation.
