# M018 Implementation Disposition — Exact Wire-Contract Reconciliation

Status: corrective pass required

Frozen initial implementation head: `ea35de9be339fa2c963f9c553cbbcf01540e3ee3`

Initial disposition commit: `db5e0679369dcefe61eb24bd10079dbf98086cea`

Implementation executor: Codex implementation run recorded in the repository session that produced `ea35de9`.

This record preserves the implementation evidence that landed at `ea35de9`, but its prior `closing` disposition is no longer sufficient for final review. A later internal source-level review found unresolved protocol-semantic and coverage-classification defects. M018A is now the active corrective handoff; M019 is superseded and M019A remains blocked.

## Internal-only authority boundary

This workstream is internal to `eggstack/emissary`.

No implementation or review record authorizes:

- an upstream issue, pull request, merge request, discussion, review request, or patch submission;
- upstream approval, adoption, or merge solicitation;
- branch, commit, tag, patch, or artifact pushes to an upstream remote;
- upstream maintainer outreach;
- preparation of an upstream contribution package or merge plan.

External Proposal 170 and reference-implementation sources are read-only evidence. All repository writes must remain in `eggstack/emissary` unless a future explicit maintainer directive supersedes this policy.

## Retained implementation evidence

The following initial M018 work remains retained unless M018A finds a direct regression:

| Requirement | Evidence | Current disposition |
|---|---|---|
| Exact 43 RouterInfo addition strings | `rpc::router_info_keys::PROPOSAL_170_ADDITIONS` and exact-set tests | retained |
| Canonical RouterInfo direct parameter presence | `router_info_handler.rs` and literal direct-form tests | retained |
| Canonical nullable fields and `logs.clear` type | focused RouterInfo fixtures | retained |
| Truthful unavailable canonical fields | source-state manifest and whole-request unavailable path | retained |
| Canonical AddressBook modes | `AddressBook` direct entry, `SetSubscriptions`, and `SetConfig` handlers and fixtures | retained |
| AddressBook response adjudication | linked Java reference implementation read-only comparison | retained for M019A recheck |
| Compatibility AddressBook forms | separate action-style and standalone method paths with mixed-form rejection | retained as compatibility only |
| Seven lowercase TunnelManager actions | canonical action parser and action fixture | retained |
| Canonical TunnelManager structured success responses | create/edit/get/lifecycle/delete success fixtures | retained, but failure envelopes require M018A correction |
| Tunnel option inventory | documentation matrix, typed validation, and raw round-trip | retained |
| ClientServicesInfo direct presence | direct official-form, any-value, and mixed-form fixtures | retained |
| Bounded SAM observation | M016 publisher lifecycle evidence plus closest-production composition serializer test | qualified; M019A adjudication required |
| Wire/source/runtime documentation separation | conformance and support documents | retained, subject to manifest correction |
| Scope guard | no CI, release, frontend, broad core, or tunnel data-plane edits | retained |

## Post-disposition findings

| ID | Finding | Severity | Disposition |
|---|---|---|---|
| M018A-F1 | `i2p.router.net.total.transit.bytes` returns received plus sent transit bytes instead of the forwarded/transmitted total | high | return to M018A |
| M018A-F2 | Valid canonical TunnelManager operational failures sometimes return JSON-RPC application errors instead of structured `result.status` | high | return to M018A |
| M018A-F3 | `conformance_manifest.rs` still labels base methods and action-style AddressBook compatibility modes as canonical Proposal 170 inventory | medium | return to M018A |
| M018A-F4 | TunnelManager documentation contains capitalized canonical examples and imprecise `Name`/`All` requirements | low | correct in M018A |
| M018A-F5 | Active planning lacked an absolute internal-only/no-upstream-submission rule | governance | corrected in planning governance and active handoffs; verify in M018A |
| M018-F6 | No true real-session-to-production-ClientServicesInfo test | medium evidence decision | retained qualified evidence; M019A must accept or reject explicitly |

## Why the initial verification missed these findings

- The exact-set tests checked selector names and JSON types but did not assert the semantic relationship between distinct transit received and transmitted counters.
- TunnelManager fixtures primarily covered successful canonical actions and unsupported backend statuses; they did not exhaust valid-operation lookup, ownership, persistence, and backend failure envelopes.
- The static conformance manifest was updated around exact RouterInfo and TunnelManager sets but retained historical comments and counts that conflated base/compatibility inventory with Proposal 170 canonical additions.
- Planning referenced upstream sources for contract interpretation without an explicit prohibition against upstream write/submission activity.

M018A must add regressions that would fail on each of these defects.

## SAM evidence limitation

The initial M018 pass did not claim a true end-to-end SAM session lifecycle test.

`emissary-cli/tests/production_composition.rs::production_sam_observation_source_reaches_client_services_serializer` exercises the production control adapters, shared `SamSessionObservationHandle`, `I2pControlState`, and canonical serializer with a listening SAM registry entry and a bounded empty snapshot.

M016 separately proves publisher insertion, socket updates, removal, generation, overflow, and serialization behavior. M019A must decide whether the combined evidence is adequate for internal revision-bound closure. It must not relabel this as a true live-session end-to-end test.

## Initial verification outcomes retained

The initial M018 head recorded successful package-scoped check, test, clippy, touched-file nightly rustfmt, and `git diff --check` outcomes, including 1,130 feature-gated package tests.

These results do not close M018A. The corrective implementation must rerun the targeted package commands and add focused regressions for transit semantics, TunnelManager canonical failures, and manifest classification.

## Current handoff

Active implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/018a-wire-semantics-and-internal-only-corrective-pass.md`

Blocked internal closure plan:

- `plans/implementation/i2pcontrol-proposal-170/019a-internal-pinned-revision-reclosure.md`

M018A must freeze a new complete implementation/test head and create:

- `plans/closure/i2pcontrol-proposal-170/018a-implementation-disposition.md`

Only then may M019A become ready.

The original M019 handoff is superseded and must not be executed.