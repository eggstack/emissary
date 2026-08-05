# M035 Implementation Disposition — Base Compatibility and Selector Overlap

Status: implemented; closure accepted

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/035-base-compatibility-and-selector-overlap.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Implementation commit:

- `5620cb8` — `feat: separate I2PControl compatibility modes`

Frozen implementation/test head: `5620cb8`

## Disposition

M035 is implemented within the I2PControl production boundary. RouterInfo now
selects one explicit internal request mode after authentication metadata is
removed. Direct requests retain Proposal 170 presence/source semantics;
historical nested `Selector` requests use only the retained base inventories,
truthy-boolean selection, and legacy serializers.

The three exact selector overlaps are represented by a literal mode-specific
table and guarded against accidental inventory intersection changes:

- `i2p.router.news`;
- `i2p.router.addressbook.subscriptions`;
- `i2p.router.addressbook.config`.

Nested address-book metadata remains the historical array/map shape. Direct
metadata retains the canonical `{path, entries}` shape. Direct router news
continues to use the Proposal 170 unavailable disposition, while nested router
news uses the existing base source when one is available.

The method inventory records implemented base methods, Proposal 170 methods,
already-shipped compatibility aliases, and unsupported base methods. `GetKeys`,
`GetRate`, `RouterManager`, `NetworkSetting`, and `AdvancedSettings` remain
standard `METHOD_NOT_FOUND` responses. No missing base adapter, router-control
owner, alias, public status, or protocol field was added.

No core, tunnel, frontend, CI/release, or upstream files changed.

## Changed-file classification

Production and tests:

- `emissary-cli/src/i2pcontrol/rpc.rs` — method inventory, separate base/direct
  selector predicates, literal overlap serializers, and static inventory guards.
- `emissary-cli/src/i2pcontrol/router_info_handler.rs` — request-mode parsing,
  mode-specific availability checks and address-book dispatch, plus focused
  compatibility tests.
- `emissary-cli/src/i2pcontrol/address_book.rs` — shared address-book source
  adapter with explicit legacy/canonical response modes; one behavior-preserving
  validator rewrite required by clippy.
- `emissary-cli/src/i2pcontrol/server.rs` — unsupported base-method dispatch
  regression coverage.

Documentation and planning:

- `docs/i2pcontrol/README.md`
- `docs/i2pcontrol/router-info.md`
- `docs/i2pcontrol/proposal-170-conformance.md`
- `docs/i2pcontrol/proposal-170-support.md`
- active registry, roadmap, implementation handoff, and M035 closure records.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Direct mode keeps exact presence/source behavior | `direct_and_nested_routerinfo_modes_are_distinct`, direct wire fixture, direct unavailable-field suite | pass |
| Nested mode keeps historical truthy selection and base inventory | `nested_news_uses_legacy_disposition`, `nested_proposal_only_selector_is_rejected` | pass |
| Direct and nested modes cannot mix | `mixed_modes_are_rejected_before_query` and existing authentication sanitization tests | pass |
| Every exact overlap has a mode-specific disposition | `ROUTER_INFO_SELECTOR_OVERLAPS`, `overlap_table_contains_every_exact_name_intersection` | pass |
| Nested news uses legacy source; direct news uses Proposal 170 disposition | `nested_news_uses_legacy_disposition`, `direct_news_uses_p170_disposition` | pass |
| Nested/direct address-book metadata shapes remain distinct | `nested_addressbook_metadata_uses_legacy_shape`, `canonical_address_book_shapes_are_literal_and_requested_only` | pass |
| Method support claim matches protected dispatch | `SUPPORT_INVENTORY`, `PROTECTED_DISPATCH`, dispatcher inventory tests | pass |
| Unsupported base methods stay standard errors | `unsupported_base_methods_return_method_not_found` | pass |
| Literal Proposal 170 fixtures remain unchanged | conformance manifest, golden fixtures, M027 literal fixtures | pass |
| No new public alias/status/protocol or out-of-bound owner | frozen changed-path review and static inventory guards | pass |

## Failure, recovery, and contention review

- Authentication metadata is removed before RouterInfo mode selection; a token
  cannot become a selector.
- Mixed-mode and unknown-selector validation complete before source queries.
- Direct unavailable selectors fail before assembly and cannot fall through to a
  base source with a different semantic disposition.
- Compatibility source failures use the existing base error path; no canonical
  source fallback is attempted.
- Existing response-size and per-source bounds are unchanged.
- No new lock, task, persistence, or network I/O was introduced.

## Compatibility and security review

- Existing direct Proposal 170 clients retain exact key-presence behavior and
  canonical address-book response shapes.
- Historical nested clients regain mode-specific behavior for the exact
  overlaps without accepting Proposal 170-only selectors.
- Unsupported methods remain sanitized standard `METHOD_NOT_FOUND` responses.
- No mutating selector is enabled by overlap handling; the only RouterInfo
  mutation remains the retained log-clear selector in direct mode.
- No source/path/token details are added to errors.
- Existing authentication, notification, request-ID, and token behavior is
  retained and covered by the package suite.

## Unresolved findings

No unresolved M035 high or medium compatibility, security, ownership, or scope
finding remains. The 26 unavailable Proposal 170 RouterInfo sources and ten
unsupported tunnel families remain explicit roadmap limitations. M036's
authentication comparison/throttling and publication durability findings are
future-plan work, not M035 defects.

## Internal-only attestation

The pinned Proposal 170 page was accessed read-only for method and selector
contract confirmation. No upstream or third-party issue, pull request, review,
submission, adoption request, maintainer contact, or connector write was
created. The commit and push directive authorizes publication of this internal
`eggstack/emissary` repository branch only.
