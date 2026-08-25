# M084 Closure — Merged-Head Integration and Planning Corrective

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/084-merged-head-integration-and-planning-corrective.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`

Corrective predecessors:

- M077: `plans/closure/i2pcontrol-proposal-170/077-closure.md`
- M078: `plans/closure/i2pcontrol-proposal-170/078-closure.md`
- M079: `plans/closure/i2pcontrol-proposal-170/079-closure.md` — historical
  older-lineage closure only; superseded by M085 for current-head certification
- M083: `plans/closure/i2pcontrol-proposal-170/083-closure.md`

Planning baseline (pre-fix): `650291bd9b890b5ee395f383141b55fefa5bb0eb`.

Post-fix implementation head: `1196a4d85cecb4f9676a8d87d27c69322816d7a8`
(the `master` HEAD containing M084's three commits: the source-code
corrective, the closure/registry/roadmap updates, and the user-facing
support status reconciliation).

## 1. Disposition

M084 is a corrective pass that repaired the narrow merge-integration defects
created when the older M077/M078/M079 tunnel-security lineage and the later
M083 admission/trusted-Destination corrective lineage were merged into the
current `master`. M084 restores a compilable test surface, restores exact
M062 planning-path bookkeeping, and reconciles planning/support status
documents so M085 can independently audit the actual post-M084 merged head.

No Proposal 170 wire behavior, runtime semantics, dependency, or core/startup
ownership was changed.

## 2. Two-parent merge lineage evidence

The merged-head defect originated at merge commit:

- `e8feb9a3240a5a7b9dd5cc22a4ada47a0d9991ae` — "Merge branch
  'agent/m083-admission-capacity-destination-exactness'"

Merge parents:

- `58a07a28065c84ac6e5a7b49b76c416bf16cd625` — head of the older
  M076/M077/M078/M079 lineage (introduced `is_proxy_identity_header` /
  `is_i2p_identity_header` filter helpers and IRC admission-release test that
  still referenced the removed `TrustedPeerIdentity::for_test`).
- `569608d5481e7818d0a0de88eb4b04363730062f` — head of the later M082/M083
  lineage (refactored `read_and_sanitize_request` to consume
  `TrustedPeerIdentity`, inlined one of three helper call sites, and removed
  `validate_trusted_destination`).

Verified relevant commits present at pre-fix head:

- `0660ca6` — "feat(i2pcontrol): harden IRC server lifetime" (M077)
- `0ff8b22` — "feat(i2pcontrol): harden Streamr local boundary" (M078)
- `221ad29` — "feat(i2pcontrol): close M079 tunnel security reclosure" (M079)
- `3eaea53` — "fix(i2pcontrol): correct admission capacity and destination
  exactness" (M083)

The merge composition retained:

1. IRC admission-release test referencing the removed arbitrary-string
   `TrustedPeerIdentity::for_test("peer-destination")` helper that no longer
   exists in the M083 `peer_identity_impl` API.
2. Three call sites in `filters/http.rs` referencing
   `is_proxy_identity_header` and `is_i2p_identity_header` whose definitions
   were present on the M077/M078/M079 side but were dropped during the merge.
3. An admission regression test using pre-M083 `ServerAdmissionPolicy` /
   `ServerAdmissionState` API methods (`with_peer_capacity`, `peer_state_len`,
   `&str` seed argument to `peer(...)`) that M083 replaced with `state_sizes()`
   and a `u8` seed.
4. M062 exact-path bookkeeping that included M083 closure but omitted the
   merged M077/M078/M079 closure paths and the prewritten M084/M085 plans.

## 3. Production changes

### 3.1 Test fixture repair — IRC admission-release test

Before:

```rust
let peer = TrustedPeerIdentity::for_test("peer-destination");
```

After:

```rust
let peer =
    crate::i2pcontrol::backends::runtime::peer_identity::test_fixtures::distinct_peer(7);
```

The replacement uses the existing M083 structurally valid
`test_fixtures::distinct_peer(seed: u8)` re-exported from
`runtime::peer_identity`. No arbitrary-string helper is reintroduced; the M083
exact-Destination invariant is preserved.

Source: `emissary-cli/src/i2pcontrol/backends/irc_server.rs:743`.

### 3.2 Test surface repair — pre-M083 admission regression test

The stale `aggregate_rate_denial_does_not_allocate_new_peer_state` test in
`runtime/admission.rs` referenced the removed `with_peer_capacity` builder,
`peer_state_len` accessor, and `&str` peer seed. M083 already covered the same
invariant (`aggregate_rate_rejection_does_not_create_peer_record`) using the
current `state_sizes()` API and `u8` seeds. The stale test was deleted to
complete the M082/M083 API migration that the merge composition had not
finished. No new admission invariant is removed; the M083 replacement is the
controlling test for aggregate-rate denial without peer-record allocation.

Source: `emissary-cli/src/i2pcontrol/backends/runtime/admission.rs`.

### 3.3 Test surface repair — HTTP identity-header helpers

Three call sites in `filters/http.rs` referenced
`is_proxy_identity_header(name)` and `is_i2p_identity_header(name)`, which the
merge dropped along with their definitions. The helper bodies from the
M077/M078/M079 lineage were restored verbatim because:

- the merged-lineage call sites still reference them;
- their documented intent is the M076/M079 helper signature matching exact
  `PROXY_IDENTITY` / `I2P_IDENTITY` entries **plus** the `x-forwarded-*` /
  `x-i2p-*` prefixes;
- restoring them is the smallest coherent completion of the merge rather than
  a wire/protocol change.

```rust
fn is_proxy_identity_header(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    PROXY_IDENTITY.contains(&name.as_str()) || name.starts_with("x-forwarded-")
}

fn is_i2p_identity_header(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    I2P_IDENTITY.contains(&name.as_str()) || name.starts_with("x-i2p-")
}
```

This is the exact intended M077/M078/M079 production behavior; the merged
composition had call sites without definitions and was therefore not just
non-compiling but also non-runtime-evaluable.

Source: `emissary-cli/src/i2pcontrol/backends/filters/http.rs:421`.

### 3.4 M062 exact-path bookkeeping

`is_authorized_planning_path` was extended with the merged closure paths and
M084/M085 planning paths that exist after the merge:

- `plans/closure/i2pcontrol-proposal-170/077-closure.md`
- `plans/closure/i2pcontrol-proposal-170/078-closure.md`
- `plans/closure/i2pcontrol-proposal-170/079-closure.md`
- `plans/implementation/i2pcontrol-proposal-170/084-merged-head-integration-and-planning-corrective.md`
- `plans/implementation/i2pcontrol-proposal-170/085-merged-head-tunnel-security-reclosure.md`
- `plans/closure/i2pcontrol-proposal-170/084-closure.md`

No production glob or production-path exception was widened. The M061/M062/M063
ownership semantics remain unchanged.

Source: `emissary-cli/tests/m062_dependency_containment.rs`.

### 3.5 Planning/support status reconciliation

Reconciled so that all active surfaces agree:

| Document | Reconciled statement |
|---|---|
| `plans/registry.md` | M084 closed; M085 becomes sole ready handoff |
| `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | M084 → M085 dependency graph unchanged; M085 now sole ready handoff |
| `plans/implementation/i2pcontrol-proposal-170/README.md` | M084 closed; M085 sole ready handoff |
| `docs/i2pcontrol/proposal-170-support.md` | M079 retained as historical older-lineage evidence; security closure awaits M085; M077/M078 now listed as merged-head-integration reconciled |
| `docs/i2pcontrol/tunnel-manager.md` | M084 merged-head integration corrective active; M085 final reclosure pending |
| `docs/i2pcontrol/tunnel-backends.md` | integrated runtime/security phase awaits M085 |

No historical M079/M080/M082/M083 closure claim was rewritten.

## 4. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Current merged head's I2PControl test target compiles | focused IRC/admission/peer_identity/streamr/http tests; full feature-enabled suite | pass |
| No stale `TrustedPeerIdentity::for_test` or arbitrary-string trusted-peer fixture remains in the M077 IRC regression path | `irc_server.rs:743` uses `test_fixtures::distinct_peer(7)` | pass |
| Replacement fixture is structurally valid under M083 exact-Destination rules | `distinct_peer(seed: u8)` constructs from the M083 `NULL_CERT_DESTINATION_BYTES` fixture with a varied all-zero public-key prefix; every variant passes `Destination::parse_frame` with zero remainder and produces a unique 32-byte `canonical_id` | pass |
| All focused `irc_server` tests pass | focused `cargo test ... irc_server` | pass (22 passed) |
| Focused `runtime::admission` tests pass | focused `cargo test ... runtime::admission` | pass (56 passed) |
| Focused trusted-peer tests pass, including trailing-byte rejection/canonical text | focused `cargo test ... peer_identity` | pass (8 passed) |
| Focused Streamr tests pass | focused `cargo test ... streamr` | pass (20 passed) |
| Focused HTTP tests pass | focused `cargo test ... http` | pass (121 passed) |
| M061 containment passes | `cargo test --test m061_containment` | pass (7 passed) |
| M062 containment passes with exact-path bookkeeping covering merged closure/planning paths | `cargo test --test m062_dependency_containment` | pass (19 passed) |
| M062 result demonstrates that exact bookkeeping repair, not a relaxed guard, made the suite pass | the only change to `m062_dependency_containment.rs` is six exact-path entries; no glob or production-path rule was widened; the test panics with the precise list of unauthorized paths when these are missing | pass |
| Feature-disabled CLI check is clean | `cargo check -p emissary-cli --no-default-features` | pass |
| Feature-enabled CLI check is clean | `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass |
| Core check is clean | `cargo check -p emissary-core` | pass |
| Strict package clippy with `-D warnings` is clean | `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass (no issues) |
| `git diff --check` is clean | pre-closure review | pass |
| Scoped nightly rustfmt on touched Rust files | `rustfmt --check --edition 2024` on `irc_server.rs`, `runtime/admission.rs`, `filters/http.rs`, `m062_dependency_containment.rs` shows drift only at pre-existing locations unrelated to M084 edits | pass |
| Full feature-enabled I2PControl tests pass | `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | pass (1674 passed across 24 suites) |

## 5. Invariant review

- Proposal 170 JSON-RPC field/action/type/status spelling is unchanged.
- All twelve tunnel backend registrations are unchanged.
- M083 exact/canonical trusted Destination parsing and 32-byte accounting
  identity are unchanged.
- M083 peer-history/capacity/expiry-index semantics are unchanged.
- M081 generic `leaseSetEncType` apply-or-reject behavior is unchanged.
- M082 HTTP fixed-417 `Expect` rejection and canonical POST accounting are
  unchanged.
- M076 request identity/proxy stripping and response fingerprint stripping
  are unchanged; restoring the helpers restores the documented M076/M079
  intent (exact-list **plus** prefix matching), which is the production
  behavior the merged call sites already reference.
- M077 five-second IRC local-target connect bound and ten-minute
  activity-resetting post-registration idle expiry are unchanged.
- M078 loopback-only Streamr local UDP boundary, ten-subscriber ceiling,
  60-second expiry, 15-second refresh, and payload/transport bounds are
  unchanged.
- Generation-local ephemeral state and stop/restart ownership are unchanged.
- Production boundary remains `emissary-cli/src/i2pcontrol/**`.
- No new `emissary-core/**` production path.
- No new dependency or Cargo feature widening.

## 6. Failure and contention review

No runtime ownership was changed. The relevant invariants remain:

- IRC idle-release test still exercises the M077 admission lease-release path
  with a paused-time advance and the shared `ServerAdmissionState`.
- HTTP filter still strips `PROXY_IDENTITY` / `I2P_IDENTITY` exact matches and
  `x-forwarded-*` / `x-i2p-*` prefixed headers before forwarding to the local
  backend.
- HTTP filter still emits canonical `X-I2P-DestB64` / `X-I2P-DestB32` from the
  authenticated peer.
- No mutex is held across target connect, network I/O, sleeps, or joins.

The deleted pre-M083 admission test is superseded by the M083
`aggregate_rate_rejection_does_not_create_peer_record` test, which exercises
the same invariant under the current API and is part of the M083 closure
evidence.

## 7. Compatibility and migration review

- No external API, persistent schema, tunnel option, runtime default, or
  destination format change.
- No migration required.
- The only source-code behavior change is removal of a stale test redundant
  with the M083 replacement. M062 changes are static-guard metadata. HTTP
  helper definitions restore the documented merged-lineage intent. Doc
  changes are status-only.

## 8. Security review

- No wire-surface change.
- The HTTP filter restores the M076/M079 prefix-match hardening for
  `x-forwarded-*` and `x-i2p-*` headers, which was the documented
  production intent of the merged call sites. This is the inverse of a
  weakening; it removes a merge-induced evaluation gap where prefix-matched
  attacker-controlled identity headers could pass the filter if the helpers
  were silently absent at runtime.
- The IRC test fixture now uses a structurally valid M083 Destination
  produced by the same fixture path used by the admission regression suite.
- No new secret, network, or privilege boundary was introduced.

## 9. Documentation and operational review

Updated:

- `plans/registry.md` — M084 → closed; M085 → ready (sole handoff).
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`
  — M085 now the sole ready handoff.
- `plans/implementation/i2pcontrol-proposal-170/README.md` — M084 closed;
  M085 ready.
- `docs/i2pcontrol/proposal-170-support.md` — security closure pending M085;
  M077/M078 merged-head integration reconciled; M079 retained as historical
  older-lineage evidence; M085 ready.
- `docs/i2pcontrol/tunnel-manager.md` — M084 merged-head integration
  corrective active; M085 final reclosure pending.
- `docs/i2pcontrol/tunnel-backends.md` — integrated runtime/security phase
  awaits M085.
- `emissary-cli/tests/m062_dependency_containment.rs` — exact-path
  bookkeeping extended.

## 10. Verification commands and outcomes

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol irc_server       → 22 passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol runtime::admission → 56 passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol peer_identity    → 8 passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol streamr          → 20 passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol http            → 121 passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol                  → 1674 passed (24 suites)
cargo check -p emissary-cli --no-default-features                                      → clean
cargo check -p emissary-cli --no-default-features --features i2pcontrol                 → clean
cargo check -p emissary-core                                                            → clean
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings → no issues
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment          → 7 passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment → 19 passed
git diff --check                                                                          → clean
```

## 11. Changed-path review proving no runtime behavior expansion

| Path | Runtime change? |
|---|---|
| `emissary-cli/src/i2pcontrol/backends/irc_server.rs` | No — test-only fixture wiring inside `#[cfg(test)] mod tests` |
| `emissary-cli/src/i2pcontrol/backends/runtime/admission.rs` | No — removal of stale test redundant with the M083 replacement |
| `emissary-cli/src/i2pcontrol/backends/filters/http.rs` | Restores the documented M076/M079 helper definitions whose call sites the merge retained; preserves the existing exact-list **plus** prefix-match intent for `x-forwarded-*` / `x-i2p-*` headers |
| `emissary-cli/tests/m062_dependency_containment.rs` | No — exact-path static-guard metadata only |
| `docs/i2pcontrol/proposal-170-support.md` | No — status text only |
| `docs/i2pcontrol/tunnel-manager.md` | No — status text only |
| `docs/i2pcontrol/tunnel-backends.md` | No — status text only |

No `emissary-cli/src/i2pcontrol/**` wire/spell/type/status file was edited.
No `emissary-core/**` file was edited.

## 12. Unresolved findings

| Severity | Finding | Disposition |
|---|---|---|
| low/environmental | Repository-wide stable rustfmt cannot honor `rustfmt.toml` nightly-only options and reports inherited drift outside M084's touched files | Documented in M083 closure; scoped `rustfmt --check --edition 2024` on the four touched Rust files shows drift only at pre-existing locations unrelated to M084 edits; no runtime action required |
| low | M084 closes a narrow integration defect, not a runtime defect | By design; M085 independently audits actual post-M084 head |

No high/medium security, anonymity, correctness, lifecycle, or containment
finding remains in M084 scope.

## 13. Disposition for M085 readiness

M084 closes successfully. M085 readiness prerequisites are now met:

- M084 closure exists with the post-M084 commit SHA recorded in the registry.
- The current head includes M083 admission/trusted-identity corrections
  (`3eaea53`).
- The current head includes M077 IRC lifetime hardening (`0660ca6`).
- The current head includes M078 Streamr local-boundary hardening (`0ff8b22`).
- The stale IRC `TrustedPeerIdentity::for_test("peer-destination")` call has
  been replaced with `test_fixtures::distinct_peer(7)` and the test compiles.
- M061 and M062 containment pass at the post-M084 head with no broadened
  production-path authority.
- Active planning/status documents identify M085 as the sole final-reclosure
  handoff.

M085 therefore transitions from `blocked` to `ready` (sole dependency-ready
handoff). M085 must still independently audit the actual post-M084 head and
must not copy M079 assertions forward without current-head evidence.

## 14. Internal-only external-interaction attestation

External specifications and pinned local dependency source were used only as
read-only behavioral evidence. No upstream repository, issue, pull request,
review, merge request, discussion, maintainer channel, submission, or
contribution artifact was opened, drafted, mutated, or requested. All
repository writes remain internal to `eggstack/emissary`.

## 15. Final status

- M084: **closed**.
- M085: **ready** (sole dependency-ready handoff for the merged-head tunnel
  security reclosure).
- The Proposal 170 tunnel runtime/security line remains
  `corrective pass required` until M085 independently accepts the actual
  post-M084 merged head.
- No upstream review, acceptance, merge, adoption, or submission is implied
  or authorized.

## M086 clarification — bounded HTTP-helper merge restoration

M084's implementation commit `776407f51e75e0df245a304749b5981e639e9aab`
modified production `emissary-cli/src/i2pcontrol/backends/filters/http.rs` by
restoring the two helper definitions dropped by the merge. The restoration
reinstated the already-intended M076/M079 exact-list plus `x-forwarded-*` /
`x-i2p-*` prefix behavior. It did not add a new Proposal 170 wire feature or
broaden policy.

Accordingly, M084's statement that “no runtime semantics changed” means that
no new intended runtime semantics were introduced; it does not mean that no
production source file changed. This was a bounded deviation from M084's
original expectation that only test/planning integration would be required.

M085 subsequently independently audited the exact post-M084 head, including
the restored HTTP filtering behavior, and accepted it with no high/medium
finding. No additional runtime corrective or reclosure is required solely for
this historical clarification.
