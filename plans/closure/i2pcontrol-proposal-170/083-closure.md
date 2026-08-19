# M083 Closure — Admission Capacity Semantics and Trusted Destination Exactness Corrective

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/083-admission-capacity-and-trusted-destination-exactness-corrective.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`

Corrective predecessor closures:

- M080: `plans/closure/i2pcontrol-proposal-170/080-closure.md` — historical
  closure retained; current-head capacity and expiry-index defects are closed
  here.
- M082: `plans/closure/i2pcontrol-proposal-170/082-closure.md` — direct HTTP
  `Expect`/POST-key corrections retained; inherited trusted-Destination
  exactness is closed here.

Planning production baseline: `a35d2bc333ff0e8b9889cd133d8ef75a98faa049`.

Implementation commit: `3eaea53`.

## 1. Executive finding

M083 is complete at the current implementation head. The shared I2PControl
accepted-server boundary now distinguishes peer-rate history from active
concurrency cleanup, rejects historical policies whose unlimited aggregate
arrivals cannot be represented, computes capacity from the tightest checked
aggregate-window bound with fixed-window overlap, and enforces an explicit
inactive-peer expiry-index invariant. Trusted peer text is decoded and parsed
with `Destination::parse_frame`, requires an empty parser remainder, and is
stored/forwarded only as canonical I2P Base64 derived from the parsed bytes.

The correction remains inside `emissary-cli/src/i2pcontrol/`; no core
production path, dependency, Proposal 170 wire surface, or upstream channel
was added.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| Minute/hour/day peer history is explicit | `ServerAdmissionPolicy::peer_history: Option<Duration>` and `every_historical_peer_window_requires_a_finite_aggregate_bound` | pass | No duration comparison is used to infer history. |
| Unlimited aggregate arrivals are rejected for unbounded historical state | `every_historical_peer_window_requires_a_finite_aggregate_bound` | pass | Covers minute, hour, and day peer windows before state construction. |
| No-history inactive churn does not accumulate | `no_history_fresh_peer_churn_does_not_accumulate_inactive_state` | pass | Final lease drop removes the peer immediately; active records remain bounded by concurrency. |
| All aggregate windows contribute to capacity | `tightest_aggregate_window_not_field_precedence_controls_capacity` | pass | Hour/day-tight and minute-tight intersections are covered. |
| Fixed-window boundary overlap is safe | `fixed_window_boundary_overlap_is_included_in_capacity_bound`; `aggregate_event_bound` | pass | Uses `limit * (ceil(history/window) + 1)` and paused Tokio time. |
| Capacity arithmetic cannot wrap downward | `checked_capacity_math_never_wraps_downward` | pass | Checked multiplication/addition returns an incoherent policy on overflow. |
| Reference defaults remain representable | `defaults_and_global_limit_are_finite`; `capacity_derivation_accepts_default_and_rejects_unrepresentable_aggregate` | pass | The conservative default requirement fits `MAX_PEER_ENTRIES`. |
| Genuine over-ceiling policies fail before allocation | `capacity_derivation_accepts_default_and_rejects_unrepresentable_aggregate` | pass | Unlimited historical and oversized aggregate policies reject at construction. |
| Expiry index has one bounded invariant | `State::assert_invariants`; `active_peer_past_expiry_remains_bounded_and_is_reindexed_on_final_drop`; `repeated_reap_is_idempotent_for_inactive_history` | pass | Inactive historical peers have one entry; active peers are intentionally unindexed. |
| Reap uses the authoritative `(deadline, key)` entry | `State::reap` and repeated reap regression | pass | No key-only reconstruction or active-peer orphaning remains. |
| Acquire/drop/reap/restart/denial transitions remain bounded | admission regression suite, including restart, denial, and expiry-index churn tests | pass | Denials remain side-effect-free apart from bounded reaping. |
| Exactly one supported Destination is required | `TrustedPeerIdentity::from_destination_text` uses `Destination::parse_frame` and checks `rest.is_empty()` | pass | Core parser was not changed. |
| Trailing Destination bytes are rejected | `rejects_one_or_many_trailing_destination_bytes` and full admission suite | pass | Both one-byte and arbitrary trailing payloads fail before admission/HTTP. |
| Canonical full-Destination text and stable ID are exposed | `accepts_supported_destinations_and_stores_canonical_text`; `canonical_id_is_stable_for_the_same_exact_destination` | pass | Canonical text and the 32-byte ID derive from the same parsed bytes. |
| Redaction and malformed-input behavior remain intact | existing debug/redaction tests plus malformed trusted-identity regressions | pass | Destination text and ID remain absent from `Debug` output. |
| HTTP receives the canonical identity and retains M082 protections | full I2PControl suite; HTTP-filter/server suite | pass | `X-I2P-DestB64`/B32, POST canonical key, fixed 417, spoof stripping, and response filtering remain green. |
| M081, M080, containment, and scope boundaries remain green | full suite, M061/M062 tests, `cargo check -p emissary-core`, and changed-path review | pass | No `emissary-core/**` production change or new dependency. |

## 3. Production implementation evidence

`emissary-cli/src/i2pcontrol/backends/runtime/admission.rs` now stores an
optional peer-history horizon. Historical policies use a checked per-window
event bound, select the minimum enabled aggregate bound, add the configured
concurrency margin, and reject unrepresentable capacity before admission
state exists. No-history policies retain only active peer records and remove
them on final lease drop.

The expiry index is explicitly an inactive-historical-peer index. Admission
removes a historical peer's entry when it becomes active; final drop creates
exactly one authoritative entry, and `reap` removes the exact entry it
observed. Active peers that outlive a nominal historical deadline remain in
the bounded peer map without an expiry entry until final drop.

`emissary-cli/src/i2pcontrol/backends/runtime/peer_identity_impl.rs` applies
the existing textual bound and input hygiene, decodes once, parses with the
existing core `Destination::parse_frame`, rejects non-empty remainder, derives
the 32-byte ID from the parsed Destination, and canonicalizes the stored full
Destination with the existing I2P Base64 encoder.

## 4. Verification executed

### Commands run

```bash
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol admission
cargo test -p emissary-cli --no-default-features --features i2pcontrol http
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
rustfmt +nightly --edition 2024 --check emissary-cli/src/i2pcontrol/backends/runtime/admission.rs emissary-cli/src/i2pcontrol/backends/runtime/peer_identity_impl.rs
```

### Results

| Command | Outcome |
|---|---|
| full I2PControl test suite | 1,642 passed across 24 suites |
| focused `admission` tests | 62 passed |
| focused `http` tests | 115 passed |
| feature-disabled CLI check | clean; 0 crates compiled |
| feature-enabled CLI check | clean |
| core check | clean; 0 crates compiled |
| feature-enabled all-target clippy with `-D warnings` | clean |
| M061 containment | 7 passed |
| M062 dependency containment | 19 passed |
| nightly rustfmt check on touched Rust files | clean |
| `git diff --check` | clean before closure-document edits; rerun for final head |

The repository-wide stable `cargo fmt --all -- --check` reports pre-existing
formatting drift outside the M083 files because the workspace uses nightly-only
rustfmt options. The scoped nightly formatter was run on both touched Rust
files and is the accepted formatting evidence for this corrective.

## 5. Invariant review

- Proposal 170 wire fields, methods, statuses, tunnel types, and option
  meanings are unchanged. `0` remains unlimited for each configured rate.
- The accepted-server architecture, M080 transactional denial behavior,
  32-byte accounting identity, bounded task ownership, M081 option truthfulness,
  and M082 `Expect`/POST-key behavior are retained.
- Historical state is retained only for enabled peer-rate semantics, while
  no-history inactive records are removed promptly.
- Every enabled aggregate limiter contributes to the capacity proof, and the
  fixed-window overlap term prevents under-budgeting.
- The peer map and expiry index remain bounded by the hard ceiling; active
  peers are bounded by concurrency and inactive historical peers have exactly
  one authoritative expiry registration.
- Trusted identity is authenticated from SAM/Yosemite, structurally parsed,
  exact, canonicalized, and redacted in diagnostics.
- HTTP continues to use the shared identity path; no local target is selected
  from remote datagram or untrusted identity text.
- Production changes are limited to the I2PControl runtime boundary. No new
  dependency or core production path was introduced.

## 6. Failure and recovery review

Construction rejects invalid or unrepresentable policy before admission state,
session, or task allocation. Admission denial paths do not create peer records
or expiry entries. Repeated reap is idempotent, active peers that outlive a
nominal expiry remain represented in the peer map, final drops restore the
documented inactive/index state, and a new runtime generation starts empty.

Malformed, truncated, whitespace/control-containing, and trailing-byte
Destination inputs fail before admission and before HTTP request construction.
Existing HTTP fixed-error, local-connect ordering, proxy-stripping,
fingerprint-filtering, and POST limiter regressions remain green in the full
and focused suites.

## 7. Migration and compatibility review

The change is internal to I2PControl admission and trusted-peer plumbing. It
does not alter Proposal 170 JSON-RPC names or configured rate meanings. The
reference default's internal conservative capacity proof includes the documented
boundary margin. Valid trusted Destination identities retain their canonical
identity ID; full-Destination text is normalized to canonical Base64, so
attacker-selected textual aliases do not reach downstream access/header paths.

No persisted schema, private destination material, startup ownership, Yosemite
contract, or core parser behavior changed. Restart creates fresh bounded
admission state as before.

## 8. Security review

The corrected capacity proof prevents unlimited historical identity churn and
avoids false acceptance based on a duration-equality shortcut. The no-history
cleanup rule removes avoidable table occupancy. Checked arithmetic and the hard
ceiling prevent downward-wrap acceptance. Exact Destination consumption blocks
trailing-payload identity confusion, while canonical text keeps HTTP/access
matching aligned with the authenticated bytes. Debug output and errors remain
redacted. No new secret, network, or privilege boundary was introduced.

## 9. Documentation and operations

Updated planning and support records:

- `plans/registry.md` — M083 closed; M080/M082 current dispositions reconciled;
  M077 advanced to `ready`; M078/M079 remain blocked.
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`
  — current baseline and dependency sequence advanced.
- M080 and M082 implementation-plan statuses — corrective history recorded.
- `docs/i2pcontrol/proposal-170-support.md`,
  `docs/i2pcontrol/tunnel-backends.md`, and
  `docs/i2pcontrol/tunnel-manager.md` — M083 closure and M077 sequencing.
- `emissary-cli/tests/m062_dependency_containment.rs` — M083 closure path
  admitted to the internal planning allowlist.

## 10. Unresolved findings

None at M083 scope. M080 is closed with corrective history: its historical
transactionality/cardinality closure remains valid and M083 closes the
remaining current-head capacity and expiry-index findings. M082 is closed with
corrective history: its direct HTTP fixes remain valid and M083 closes the
inherited trusted-Destination exactness finding. No high or medium finding
remains in the M083 scope.

## 11. Roadmap disposition

M083 is closed and the next hard dependency is unblocked. M077 is now the sole
dependency-ready implementation plan because it consumes the corrected shared
admission/trusted-peer boundary. M078 remains blocked behind M077, and M079
remains blocked behind M077 and M078 as the final independent tunnel-security
reclosure authority.

## 12. Registry updates

The active registry now records:

- the tunnel-security roadmap as active with M083 closed and M077 ready;
- M080 and M082 as closed with corrective history;
- M083 as closed with this closure record;
- M077 as the sole ready handoff;
- M078 and M079 as blocked by their named sequencing dependencies.

## 13. Internal-only boundary

External specifications and pinned local dependency source were used only as
read-only behavioral evidence. No upstream repository, issue, pull request,
review, merge request, discussion, maintainer channel, submission, or
contribution artifact was opened, drafted, mutated, or requested. All
repository writes remain internal to `eggstack/emissary`.
