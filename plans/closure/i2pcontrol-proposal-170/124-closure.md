# M124 — Corrected Yosemite Y005 Auth-Consistency Pin Adoption Closure

Status: **closed**

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/124-y005-auth-consistency-pin-adoption.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md`

Repository baseline reviewed: `045d1e8b4eba1141d2488882f99c5ce994db91a8`

Implementation commits:

- `8a302b0` — adopt Yosemite Y005 through the optional I2PControl alias and add
  dependency-boundary regressions.
- `d4ffab2` — records the M124 closure and planning-state reconciliation.

## 1. Executive finding

M124 is closed. Emissary now exact-pins Yosemite Y005
`59140a2277bf296928d2e8ce39a148182eeff044` only through the optional
`yosemite-i2pcontrol` alias. The ordinary workspace Yosemite dependency remains registry
`0.7.0`. Y005's cross-field LeaseSet auth validation is reachable through I2PControl-owned
tests: coherent DH and PSK configurations serialize only their selected namespace, while
mismatched mode/type configurations fail before session allocation. No Proposal LeaseSet mapping,
router behavior, cryptography, or support-matrix promotion was added.

## 2. Readiness gates

All hard gates were satisfied before adoption:

1. Yosemite Y005 is closed in its own repository at
   `59140a2277bf296928d2e8ce39a148182eeff044`, and its closure explicitly marks the revision
   suitable for future Emissary consumer pinning.
2. Y005 reports no unresolved high/medium protocol or security finding in the consumed
   option/serialization surface.
3. The reviewed Y004→Y005 production diff is Yosemite-generic and limited to
   `src/options.rs` and `src/proto/session.rs`; no dependency or feature change occurred.
4. M123 is closed with no open high/medium server-transaction finding.
5. Emissary retains the ADR-0005 optional alias boundary and has no workspace-wide Yosemite
   patch, path replacement, vendored copy, or floating fork reference.

## 3. Exact Y004→Y005 diff review

The read-only comparison of Y004
`c2db73dba35dd9392947af5c74df29b0b556775f` to Y005
`59140a2277bf296928d2e8ce39a148182eeff044` was reviewed from the authorized Yosemite checkout.
The production diff is 182 additions and 41 deletions in exactly two files:

- `src/options.rs` — adds the cross-field invariant tying LeaseSet type, auth type, and each
  typed DH/PSK client entry. It preserves the existing public types, constructors, validation
  domains, redaction, and default values.
- `src/proto/session.rs` — derives one selected auth namespace from the validated auth type and
  serializes only that namespace with the existing deterministic numbering. Tests add the
  truth-table regressions and repair the former mixed-mode positive fixture.

No public API shape, dependency, feature, default behavior, or generic option vocabulary changed.
The Y005 diff contains no Emissary, I2PControl, Proposal, router, cryptography, persistence, or
release concept. Yosemite's Y005 closure has no unresolved in-scope high/medium finding.

## 4. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| Exact reviewed Y005 pin | `emissary-cli/Cargo.toml`, `Cargo.lock`, and M062 manifest all contain `59140a2277bf296928d2e8ce39a148182eeff044`. | pass | Existing optional alias only. |
| Ordinary provenance unchanged | Workspace `yosemite` remains registry `0.7.0`; disabled tree has no fork source. | pass | No `[patch]`, path, vendor, or replacement. |
| Coherent DH behavior | `m124_y005_coherent_leaseset_wire_is_reachable_at_fake_sam` observes DH namespace and rejects PSK namespace. | pass | Actual `yosemite_i2pcontrol` alias. |
| Coherent PSK behavior | Same test observes PSK namespace and rejects DH namespace. | pass | Actual fake-SAM `SESSION CREATE` bytes. |
| Mismatched mode/type rejection | `m124_y005_malformed_leaseset_values_reject_before_wire` covers wrong mode, mixed mode, no-auth entries, and non-type-5 auth. | pass | InvalidOption occurs before a connection can be made. |
| Historical namespace safety | Tests assert canonical private/signing names and absence of `i2cp.leaseSetClientAuth`; M122/Y004 closure records remain unchanged. | pass | No legacy non-canonical namespace returned. |
| Secret/debug hygiene | Existing LeaseSet redaction regression remains green; invalid errors are generic and no fixture secret is logged. | pass | No secret-bearing production path added. |
| Containment/provenance | M062 guard, import enumeration, and both cargo-tree checks pass. | pass | 16 production import files are all below `emissary-cli/src/i2pcontrol/`. |
| Proposal state unchanged | M095/M105 tests pass; matrix remains 284/98/458 and no Proposal mapping changed. | pass | LeaseSet cells remain blocked. |

## 5. Production and dependency implementation evidence

The only Emissary manifest change is the `rev` on the pre-existing optional
`yosemite-i2pcontrol` alias. Cargo updated only the corresponding fork source references in
`Cargo.lock`; the registry Yosemite package remains present and unchanged.

Emissary production behavior was not expanded. The only source change is test/helper code in
`emissary-cli/src/i2pcontrol/backends/runtime/session.rs`, used to exercise the dependency's
generic session serializer. No `TunnelOptions` or Proposal field is translated to LeaseSet auth.

## 6. Verification executed

### Commands run

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo tree -p emissary-cli --no-default-features --edges normal
cargo tree -p emissary-cli --no-default-features --features i2pcontrol --edges normal
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

### Results

- Both package checks passed, including feature-disabled and default workspace builds.
- The I2PControl library suite passed: 709 tests.
- The full I2PControl package suite passed: 1,903 tests across 26 suites.
- The focused M061/M062/M095/M105 command passed: 33 tests.
- Clippy passed with `-D warnings`.
- The disabled dependency tree contains only registry Yosemite. The enabled tree contains
  registry Yosemite plus exactly one fork instance at Y005.
- `git diff --check` passed.
- `cargo fmt --all -- --check` remains non-zero because the installed stable formatter cannot
  apply this repository's nightly-only settings and reports pre-existing formatting drift across
  unrelated files. No M124-added line requires formatter correction, and no formatter churn was
  introduced.

## 7. Invariant review

- Ordinary Yosemite provenance is unchanged; only the feature-gated alias moved from exact Y004
  to exact Y005.
- The alias remains optional, activated only by `i2pcontrol`, and exact `git + rev` pinned.
- All 16 production fork import files remain under `emissary-cli/src/i2pcontrol/**`.
- No workspace patch, path dependency, vendoring, global replacement, raw SAM construction, or
  non-I2PControl production change was introduced.
- Y005 capability evidence did not promote any Proposal support cell.
- Y003/Y004 historical closure records remain untouched.
- No upstream or third-party write occurred.

## 8. Failure, recovery, and contention review

M124 adds no task, timer, lock, persistent state, runtime owner, migration, or contention path.
Y005 validation is synchronous and side-effect free. Invalid direct option combinations are
rejected as `InvalidOption` before the session can connect or produce `SESSION CREATE` bytes;
there is therefore no partial runtime state or rollback state to recover. The fake-SAM positive
tests use a local loopback listener only and terminate their sessions deterministically.

Rollback is limited to restoring the exact Y004 alias revision and corresponding lock/evidence
entries. No data-format migration is required.

## 9. Compatibility and migration review

The ordinary registry Yosemite dependency and all non-I2PControl consumers retain their prior
source. Default `SessionOptions` wire behavior remains unchanged by Y005. Canonical Y004
LeaseSet names, DH/PSK value construction, numbering, generic collision checks, and redaction
remain valid. The only changed dependency behavior is fail-closed rejection of typed auth
material that the reference would ignore. No Emissary configuration or persisted data format
changed.

## 10. Security review

Y005 prevents a typed DH/PSK entry from being silently ignored under the wrong auth branch or
LeaseSet type. Emissary retains fail-before-allocation handling for unsupported Proposal
LeaseSet options and adds no downgrade or filtering behavior. Fixture keys and secrets remain
redacted from `Debug` and generic errors; no secret is included in logs or wire assertions
beyond the deliberate local fake-SAM test observation. Authentication, authorization, local
target, Streamr, router, and TLS boundaries are unchanged.

## 11. Documentation and operations

Updated:

- exact dependency and lockfile containment evidence;
- implementation README, active registry, both relevant roadmaps, and TunnelManager dependency
  documentation;
- this closure record and the M124 plan status.

The registry now records M124 closed and authorizes the focused M113 LeaseSet capability/
crypto-ownership audit as read-only planning work. It does not register a successor
implementation plan.

## 12. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | Stable/nightly rustfmt configuration drift is pre-existing and affects unrelated files. | Formatting check remains non-zero under the installed stable toolchain. | Use the repository-compatible formatter/toolchain in a future tooling pass; do not introduce unrelated churn. |
| high/medium | None. | No open M124 correctness, security, containment, or compatibility finding. | None. |

## 13. Roadmap disposition

Milestone closed and the next focused audit may proceed. M124 removes the Yosemite dependency
blocker, but it does not make M113 LeaseSet capability implementation ready. The read-only
M113 capability/crypto-ownership audit is authorized and not yet registered. M114 and the
remaining 98 applicable blocked matrix cells remain blocked.

## 14. Future-plan readiness audit

- Yosemite Y005 is closed; no later Yosemite plan is blocked on it.
- M124 is closed; no registered Emissary implementation plan becomes ready solely from this
  infrastructure adoption.
- The focused M113/LeaseSet capability and crypto-ownership audit may now begin as authorized
  read-only planning work.
- A future M113 successor may be registered only after the audit freezes exact LeaseSet type(s),
  cryptographic primitive ownership, server-side key/secret lifecycle, exact SAM mapping,
  no-downgrade behavior, bounded client-auth handling, NetDb semantics, and interoperability
  evidence. Until then, all 21 M113 cells remain blocked.
- The final reclosure remains blocked by the current 98 applicable residual cells.

## 15. Internal-only external-interaction attestation

Yosemite source, history, plan, and closure records were accessed read-only from the authorized
internal sibling checkout for dependency review. No Yosemite repository file was changed. All
Emissary writes were limited to the authorized internal `eggstack/emissary` repository. No
upstream or third-party repository or maintainer channel was mutated; no issue, pull request,
review, contact, submission, release, merge, adoption request, or contribution artifact was
created or requested.
