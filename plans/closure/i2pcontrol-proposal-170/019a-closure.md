# I2PControl Proposal 170 Milestone 019A — Internal Pinned-Revision Closure

Status: closed internally against pinned revision

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/019a-internal-pinned-revision-reclosure.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md#7-m019a--internal-pinned-revision-independent-reclosure`

Frozen M018A implementation/test head: `a3c4f469f4877e5ff4a0bb4230da298f0b367ed2`

Final reviewed implementation/test head: `a3c4f469f4877e5ff4a0bb4230da298f0b367ed2`

Internal review date: 2026-07-31

Implementation executor: `wr3n-ai`, the distinct Codex implementation run that produced
`a3c4f46` and the M018A disposition.

Internal reviewer: `Codex M019A independent closure run, 2026-07-31`, recorded by this
closure sequence after the M018A head was frozen. This is a distinct auditable execution
and commit sequence from the M018A implementation run.

## 1. Executive finding

The internal `eggstack/emissary` implementation matches the Proposal 170 wire contract
pinned to the official open revision created and last updated on 2026-05-20. The exact
canonical additions, request modes, actions, response keys, and result fields pass the
machine-checkable and literal fixture evidence reviewed below.

Unavailable data sources, unsupported tunnel runtimes, compatibility extensions, and the
qualified SAM evidence limitation remain explicitly classified. No unresolved high or
medium finding remains. This is an internal closure against a pinned open revision; it
does not claim upstream review, acceptance, certification, adoption, or merge.

## 2. Pinned source and semantic audit

The official Proposal 170 page was fetched read-only:
<https://i2p.net/en/proposals/170-i2pcontrol-expansion/>.

The page reports:

- title: `I2PControl Expansion`, Proposal 170;
- status: `Open`;
- created: `2026-05-20`;
- last updated: `2026-05-20`.

The source metadata and reviewed revision match the repository pin. No material proposal
change was found, so no rebase or new architecture decision was required.

## 3. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| Exact 43 RouterInfo additions and JSON types | `rpc::router_info_keys::PROPOSAL_170_ADDITIONS`, `PROPOSAL_170_CONTRACT`, `router_info_selector_count`, `router_info_selector_partition_integrity` | pass | Exact addition set has 43 unique keys; 121 base keys remain a separate inventory. |
| RouterInfo direct presence and canonical keys | `canonical_direct_wire_fixture_returns_exact_fields`, `canonical_logs_and_presence_semantics_are_literal` | pass | Any direct value selects; only requested keys appear; permitted nullable values remain distinct. |
| Truthful unavailable RouterInfo fields | `canonical_unavailable_field_is_explicit_error` and source-state manifest | pass | No partial or fabricated result is returned for unavailable/ambiguous sources. |
| `logs.clear` and transit semantics | `canonical_logs_and_presence_semantics_are_literal`, `canonical_transit_bytes_returns_forwarded_counter_only` | pass | `logs.clear` returns `"success"`; distinct received `11` and sent `22` yield transit `22`, not `33`. |
| Canonical AddressBook entry mode | `canonical_wire_fixture_mutates_entry_and_uses_result_object`, `handler_delete_presence_with_false_value` | pass | Exact `Type`, `Hostname`, `Destination`; `Delete` is selected by presence, regardless of value. |
| Canonical AddressBook subscription/config modes | `canonical_wire_fixture_supports_subscription_and_config_modes`, `canonical_address_book_modes_are_explicit` | pass | In-method `SetSubscriptions` and `SetConfig` return the adjudicated `result` envelope. |
| AddressBook compatibility separation | `canonical_and_compatibility_address_book_forms_cannot_mix`, `compatibility_address_book_actions_are_not_canonical_modes` | pass | Action-style and standalone forms remain compatibility/base surfaces and are excluded from canonical totals. |
| TunnelManager canonical action set | `canonical_wire_fixture_covers_all_seven_actions`, `tunnel_action_manifest_count`, `canonical_tunnel_actions_are_separate_from_compatibility_actions` | pass | Exactly seven lowercase actions; capitalized actions and `List` remain compatibility-only. |
| TunnelManager success/failure envelopes | `canonical_operation_failures_use_structured_status`, `handler_unsupported_never_reports_running` | pass | Valid operational failures use `result.status`; `get` uses `status` and `info`; unsupported runtimes never report running. |
| TunnelManager malformed requests | `canonical_operation_failures_use_structured_status` and existing validation tests | pass | Missing required fields and invalid parameter shapes remain JSON-RPC `INVALID_PARAMS` errors. |
| ClientServicesInfo direct presence | `canonical_direct_wire_fixture_selects_by_presence`, `canonical_and_compatibility_selectors_cannot_be_mixed` | pass | Any value selects a direct key; nested `Selector` is compatibility-only; mixed forms reject. |
| Bounded SAM observation | `serialize_sam_sessions_preserves_pinned_active_shape`, `resolve_sam_listening_without_observation_is_unavailable`, M016 publisher tests | pass, qualified | Overflow/missing source fails closed; the closest production composition seam is covered, but no true live SAM session activation test is claimed. |
| Base/canonical/compatibility classification | `emissary-cli/tests/conformance_manifest.rs` direct target, docs, and static inventory | pass | Base protocol, exact canonical additions, and Emissary extensions are separately named and counted. |

## 4. Changed-file classification

The reviewed M018A implementation scope is limited to the eight files recorded in its
disposition:

- RouterInfo metric mapping and focused tests;
- TunnelManager canonical result-envelope logic and focused tests;
- conformance manifest classification;
- directly affected Proposal 170 documentation.

The M019A review changed no production behavior. Its repository changes are closure and
planning records plus stale status/documentation corrections, including the corrected
lowercase `All` wording in `docs/i2pcontrol/tunnel-manager.md`. No CI, release, packaging,
dependency, frontend, router, transport, NetDB, tunnel data-plane, cryptographic, resolver,
SAM architecture, or I2CP redesign entered scope.

## 5. Wire, source, runtime, and evidence classification

| Surface | Wire implemented | Source available | Runtime implemented | Evidence |
|---|---|---|---|---|
| RouterInfo | Exact canonical keys/types and presence behavior | Available selectors use existing truthful sources; unavailable selectors fail explicitly | Only source-backed selectors are operational | Full focused and package evidence passed |
| AddressBook | Exact canonical modes, fields, presence, and result envelope | Persistent administrative stores | Administrative CRUD/configuration only; not runtime resolver authority | Canonical and mixed-form fixtures passed |
| TunnelManager | Seven canonical actions and structured operation results | Durable control-plane definitions | Unsupported backends remain explicit and inactive; no false running state | Success, failure, malformed, and unsupported tests passed |
| ClientServicesInfo | Six direct selectors and exact response forms | Shared live service registry and bounded SAM projection | Observed service state only; no lifecycle control | Direct-presence and production-composition evidence passed |
| Compatibility extensions | Retained and separately recognized | Existing Emissary paths | Compatibility behavior only | Excluded from canonical totals and mixed with canonical forms only when explicitly rejected |

## 6. SAM evidence disposition

The qualified SAM evidence is accepted for this bounded internal closure, with its
limitation retained explicitly. M016 proves publisher insertion, socket updates, removal,
generation fencing, bounds, overflow behavior, and serialization. The production-composition
test proves that real production controls and the canonical shared observation handle reach
the `ClientServicesInfo` serializer. A real SAM protocol activation with a live destination
and session is not deterministic in the repository-wide test environment and is not claimed.

The implementation therefore reports a genuine empty snapshot when the bounded source says
there are zero sessions, and returns an internal error on overflow or missing observation
source. This evidence decision does not authorize a new SAM plan or reopen SAM architecture.

## 7. Verification executed

### Commands run

```bash
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo test -p emissary-cli --no-default-features --features i2pcontrol transit
cargo test -p emissary-cli --no-default-features --features i2pcontrol tunnel_manager
cargo test -p emissary-cli --no-default-features --features i2pcontrol conformance_manifest
cargo test -p emissary-cli --no-default-features --features i2pcontrol client_services
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest
cargo fmt --all -- --check
git diff --check
```

### Results

- Package check passed.
- Full feature-gated package tests passed: `1137 passed` across `15 suites`.
- Clippy passed with `-D warnings` and no issues.
- Transit focused test passed: `3 passed`.
- TunnelManager focused test passed: `131 passed`.
- ClientServicesInfo focused test passed: `85 passed`.
- The plan's unqualified `conformance_manifest` filter selected no tests (`0 passed,
  1137 filtered out`) because it filters test names rather than selecting the integration
  test binary. The corrected direct target command passed: `56 passed`.
- `cargo fmt --all -- --check` remains blocked by pre-existing workspace-wide formatting
  differences and stable-rustfmt configuration incompatibilities; no Rust production files
  were changed by M019A. This is the known repository baseline limitation.
- `git diff --check` passed.

## 8. Invariant, failure, and security review

- No protocol field, alias, status, method, or tunnel type outside the pinned contract or
  documented compatibility surface was added.
- Canonical and compatibility forms reject ambiguous mixing where required.
- Administrative AddressBook state remains separate from runtime resolution.
- Unsupported tunnel backends are explicit, inactive, and never serialized as running.
- Bounded SAM collections fail closed on overflow and unavailable sources; no secrets,
  destinations, keys, payloads, or mutable session handles cross the observation boundary.
- Persistence, bounds, validation, redaction, ownership rejection, duplicate/collision
  handling, malformed requests, and deterministic unsupported outcomes remain covered by
  the retained implementation tests.
- No cancellation, restart, contention, migration, or runtime architecture scope was
  reopened by this closure-only review.

## 9. Internal-only compliance attestation

All review artifacts and repository writes targeted `eggstack/emissary`.

- No upstream issue, pull request, merge request, discussion, review request, patch,
  branch, tag, or comment was created or modified.
- No upstream maintainer was contacted for review, adoption, approval, or merge.
- No submission package, contribution checklist, patch series, or upstream merge plan was
  produced.
- The official Proposal 170 page was accessed read-only for source verification.
- The requested repository push is an internal `eggstack/emissary` repository write; no
  upstream contribution activity is part of this closure.

## 10. Findings

| Severity | Finding | Disposition |
|---|---|---|
| High | Transit total could double-count received and sent bytes | Resolved by M018A; distinct-counter regression passed. |
| High | Canonical TunnelManager operational failures could escape the structured result envelope | Resolved by M018A; failure regression passed. |
| Medium | Base and compatibility inventory could be counted as canonical | Resolved by M018A; manifest partition tests passed. |
| Medium evidence decision | No true live-session SAM-to-production end-to-end test | Accepted as a qualified evidence limitation; no full end-to-end claim is made. |
| Low | TunnelManager documentation named capitalized actions in the `All` restriction note | Corrected during M019A documentation review. |
| Governance | Active planning lacked an absolute internal-only boundary | Resolved and attested in normative governance and active planning records. |

Unresolved high findings: none.

Unresolved medium findings: none.

## 11. Roadmap, registry, and future-plan disposition

M018A remains preserved as the implementation disposition and is now closed for
implementation. M019A is closed internally against the pinned revision. M017, M018, and
their corrective/invalidation records remain historical evidence and were not rewritten as
passing final records.

The registry and roadmap now remove M019A from dependency-ready and active-closure queues,
mark the Proposal 170 subsystem `closed internally against pinned revision`, and retain the
superseded M019 handoff as non-executable history. There is no future plan in the roadmap
that is blocked and can now be unblocked, and no dependency-ready successor was created.
The Open proposal may require a new internal comparison if its revision changes; that does
not reopen this closure automatically.

## 12. Bounded internal closure statement

The internal `eggstack/emissary` implementation matches the Proposal 170 wire contract as
pinned to the reviewed open revision, with unavailable data sources, unsupported tunnel
runtimes, compatibility extensions, and evidence limitations documented truthfully.

This status is `closed internally against pinned revision`. It does not imply upstream
review, upstream acceptance, compatibility certification, adoption, intended upstream merge,
or permanent conformance to future revisions.
