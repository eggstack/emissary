# M052 — RouterInfo 26-Source Integration, Containment Review, and Reclosure

Status: corrected/closed through M056; historical closure retained

Planning baseline: `b759038`

Historical closure record: `plans/closure/i2pcontrol-proposal-170/052-closure.md`

Corrective reclosure authority: `plans/implementation/i2pcontrol-proposal-170/056-m049-m050-corrective-reclosure.md`

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Hard dependencies: M045–M051 closed or explicitly blocked with accepted semantic disposition

Milestone class: closure/evidence gate

## 1. Objective

Independently validate the complete line of work for the 26 previously unavailable Proposal 170 RouterInfo additions. M052 is validation and reclosure only; it must not patch production code.

The original M052 closure at the integrated `970252c` line accepted a 40 available / 1 protocol-permitted neutral / 2 unavailable matrix. Post-closure review subsequently invalidated three of those available claims: transit-15s request independence and both v4/v6 network-error source claims. M056 now owns the corrective integrated reclosure after M054/M055.

The target remains truthful RouterInfo behavior against the pinned Proposal 170 revision while preserving the broader repository status as `partial Proposal 170 support` for unrelated accepted unsupported dimensions.

## 2. Invariants

- No production changes in M052 or its M056 corrective reclosure. A material production defect creates a corrective implementation plan.
- Every field reported available has a named authoritative production source and exact type/shape fixture.
- A serializer, request-local cache, or manually populated test setter is not source-owner evidence.
- No literal placeholder zero/false/empty/null is accepted unless the contract explicitly permits that current/neutral value and an authoritative owner was queried successfully.
- Request history may not masquerade as router-owned rolling traffic state.
- Missing network-error authority may not serialize as code `0` / `No error`.
- Core changes remain neutral read-only/passive observation only and introduce no mutable control authority.
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

Post-closure corrective focus is limited to:

- `i2p.router.net.bw.transit.15s` through M054;
- `i2p.router.net.error` and `.error.v6` through M055;
- final source accounting through M056.

## 4. Containment audit

Compare the final corrective head against the M052 historical head and classify every new production change as:

1. I2PControl policy/adapter correction;
2. neutral core event observation within M054's `events.rs` budget;
3. error-only core scaffold cleanup within M055's `events.rs`/`inspection.rs` budget.

Any production change outside M054/M055's machine-readable budgets is a corrective blocker unless separately planned and closed. M056 itself authorizes no production change.

Specifically inspect crypto, I2NP, routing, tunnel selection/build algorithms, transport state machines, NetDB protocol behavior, proxy/UI, AddressBook, workflows, and release apparatus for unauthorized changes.

## 5. Operational evidence

Exercise the real feature-enabled child process with TLS/authentication and representative retained/corrected selectors. Pair process-level smoke evidence with deterministic owner/production-adapter fixtures for semantics that are impractical to create through a live loopback router.

Transit-15s must have deterministic evidence proving that source history advances without RouterInfo reads; a live child process returning syntactically valid zero is not sufficient. Network-error selectors must demonstrate unavailable/no-partial-result behavior unless an independently authorized owner exists.

No public-network certification or elaborate traffic-generation harness is required.

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

M056 may accept the corrected RouterInfo source matrix only when every one of the 26 target fields is either:

- operational from a truthful bounded authoritative source with exact wire and semantic evidence; or
- explicitly unavailable with an accepted semantic/owner limitation.

The pre-review `40/1/2` matrix must not be reused automatically.

Expected outcomes under current corrective plans:

- if M054 restores exact request-independent transit-15s: 38 available / 1 neutral / 4 unavailable;
- if M054 must demote transit-15s: 37 available / 1 neutral / 5 unavailable.

M056 derives the actual count from code and production evidence rather than forcing either result.

The overall Proposal 170 subsystem remains `partial Proposal 170 support` unless a separate authorized roadmap closes all other unsupported dimensions.

## 8. Stop/disposition rules

- Any high/medium correctness, security, containment, or source-truthfulness defect: another `corrective pass required`.
- Required evidence unavailable: `blocked`.
- Any semantically unavailable row: RouterInfo source completion remains incomplete with truthful partial matrix.
- No production changes are allowed under M056; required code changes produce another plan.

## 9. Closure evidence

The original M052 closure remains historical evidence and is not rewritten. M056 must create a new independent closure record with exact corrective heads, historical defect reproductions, requirement-to-evidence matrix for the three affected rows, final 43-row source audit, commands/outcomes, changed-path classification, failure/recovery/contention review, documentation reconciliation, residual findings, and explicit read-only-external/internal-only attestation.
