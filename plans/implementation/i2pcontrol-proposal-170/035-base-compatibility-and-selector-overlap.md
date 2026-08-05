# M035 — Base I2PControl Compatibility and RouterInfo Selector Overlap

Status: closed

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Applicable governance:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`

Repository baseline:

- `555d02b` — accepted M034 implementation/closure head

Hard dependency:

- M034 closed

## 1. Bounded objective

Freeze and correct the compatibility boundary between the existing/base
I2PControl API and the direct Proposal 170 expansion.

M035 must distinguish direct Proposal 170 selector-by-presence requests from the
historical nested compatibility form, resolve exact-name overlaps according to
request mode, inventory the actually implemented base methods, and prevent
unsupported base capability from being documented as complete.

M035 does not implement the entire base I2PControl API. Any missing base method
whose implementation would require a separate router-control capability becomes
explicit deferred work rather than being silently included in this milestone.

## 2. Current evidence and defect boundary

Retained M020 evidence covers authentication, token placement, standard errors,
notifications, request IDs, and a broad base RouterInfo compatibility catalog.
The protected dispatcher currently exposes a Proposal-170-focused method set and
returns `METHOD_NOT_FOUND` for other standard base methods.

The RouterInfo inventories contain exact-name overlaps, including:

- `i2p.router.news`;
- `i2p.router.addressbook.subscriptions`;
- `i2p.router.addressbook.config`.

The current handler validates requested names against a shared canonical table,
which may cause a historical nested/base request to inherit Proposal 170
availability or response-shape rules.

Proposal 170 states that existing I2PControl applications should continue to
work without modification. M035 must test the compatibility that Emissary
actually claims and stop overstating unsupported method coverage.

## 3. Required invariants

1. Direct Proposal 170 request mode retains exact current wire semantics.
2. Nested compatibility request mode retains historical truthy-boolean selection
   and legacy parameter behavior where implemented.
3. Direct and nested modes cannot be mixed.
4. Exact-name overlaps are dispatched by request mode, not by a single ambiguous
   table lookup.
5. A base request is never changed merely to make a canonical fixture pass.
6. Unsupported base methods return standard `METHOD_NOT_FOUND` and are listed
   honestly; no fabricated success or placeholder state.
7. Existing authentication/error/notification/request-ID behavior remains
   unchanged.
8. No new alias, method, selector, status, or capability-discovery extension.
9. No router algorithm, shutdown/restart manager, network setting, advanced
   setting, or rate implementation is added without a separate plan.
10. No TunnelManager, AddressBook runtime, core, frontend, CI/release, or upstream
    expansion.

## 4. Scope and file budget

Primary production scope:

- `emissary-cli/src/i2pcontrol/rpc.rs`;
- `emissary-cli/src/i2pcontrol/router_info_handler.rs`;
- `emissary-cli/src/i2pcontrol/server.rs` dispatcher only for exact inventory
  documentation/guards;
- literal compatibility fixtures and documentation.

Changes outside `emissary-cli/src/i2pcontrol/**` are prohibited unless a focused
existing base method adapter already exists and requires one trivial composition
input. Any broader base method implementation must stop and receive a separate
plan.

## 5. Target compatibility model

### 5.1 Request mode

Parse one explicit internal mode:

- `CanonicalDirect` — every supported direct key is selected by presence and
  Proposal 170 contract/source rules apply;
- `CompatibilityNested` — historical nested `Selector` truthy selection and
  historical base serializer/source rules apply.

Authentication metadata is removed before mode selection.

### 5.2 Selector inventories

Maintain separate machine-readable inventories for:

- base/legacy selector names and serializers;
- Proposal 170 additions and serializers;
- exact overlaps with a mode-specific mapping.

An overlap must appear in an explicit overlap table and tests; accidental
intersection is a compile/test failure.

### 5.3 Method inventory

Create one exact method support table:

- implemented and protected;
- compatibility alias, if already shipped;
- unsupported base method returning `METHOD_NOT_FOUND`;
- Proposal 170 method.

Do not implement `GetRate`, `RouterManager`, `NetworkSetting`, or
`AdvancedSettings` inside M035 unless the repository already contains a complete
bounded adapter and the change is only dispatch exposure. Otherwise document the
limitation and open no implicit successor plan.

### 5.4 Response shape

Overlapping address-book selectors must return the legacy shape in nested mode
and canonical `{path, entries}` shape in direct mode if both are retained.
`i2p.router.news` compatibility behavior must remain historical where a truthful
base source exists; otherwise the exact limitation must be documented without
claiming canonical availability.

## 6. Ordered work packages

### WP1 — Pin the official and internal inventories

Create literal fixtures for the base methods/selectors currently supported and
the three exact overlap names. Record current failing/ambiguous behavior.

### WP2 — Separate request-mode dispatch

Refactor only enough to pass an internal mode into validation/serialization.
Do not duplicate source queries or create a second handler implementation.

### WP3 — Add overlap table and guards

Make every intersection explicit. Add tests that fail if a new name overlaps
without a mode-specific disposition.

### WP4 — Freeze method support status

Add the exact dispatcher inventory and documentation. Unsupported base methods
remain standard errors, not partial stubs.

### WP5 — Reconcile documentation and disposition

Update RouterInfo, support, conformance, and base compatibility documentation.
Create:

- `plans/closure/i2pcontrol-proposal-170/035-implementation-disposition.md`.

## 7. Failure and contention semantics

- Unknown/mixed-mode parameters fail before source queries.
- One unavailable direct canonical selector aborts the direct request as today.
- Compatibility source failure uses the existing base error behavior and never
  falls through to a semantically different canonical source.
- No new locks, tasks, persistence, or network I/O.
- Request processing remains bounded by existing selector/response limits.

## 8. Compatibility and migration

- No persistence migration.
- Direct Proposal 170 clients remain unchanged.
- Historical nested clients regain/preserve mode-specific behavior for exact
  overlaps.
- Unsupported base methods remain unsupported but are documented accurately.
- No capability-discovery extension is added.

## 9. Security review requirements

Review and test:

- token metadata cannot be interpreted as a selector;
- mixed direct/nested requests fail;
- errors contain no internal source/path details;
- compatibility mode cannot bypass availability/authentication checks;
- no mutating selector is accidentally enabled by alias overlap;
- no upstream interaction.

## 10. Focused tests

Required semantics include:

- `direct_and_nested_routerinfo_modes_are_distinct`;
- `overlap_table_contains_every_exact_name_intersection`;
- `nested_news_uses_legacy_disposition`;
- `direct_news_uses_p170_disposition`;
- `nested_addressbook_metadata_uses_legacy_shape`;
- `direct_addressbook_metadata_uses_canonical_shape`;
- `mixed_modes_are_rejected_before_query`;
- `base_method_inventory_matches_dispatcher`;
- `unsupported_base_methods_return_method_not_found`;
- `direct_p170_literal_fixtures_remain_unchanged`.

## 11. Verification commands

```bash
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol router_info
cargo test -p emissary-cli --no-default-features --features i2pcontrol compatibility
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test golden_fixtures
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m027_literal_fixtures
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

Use targeted formatting and `git diff --check`. No CI/release/fuzz/soak
expansion.

## 12. Documentation and static guards

Add guards proving:

- inventories and overlap table are exact;
- dispatcher method list matches documented support;
- direct/nested modes cannot mix;
- no new method/selector alias entered the public surface;
- no production change outside `i2pcontrol/**`.

## 13. Acceptance criteria

M035 may close only when:

- overlap behavior is mode-specific and literally tested;
- the base method support claim matches the dispatcher;
- direct Proposal 170 fixtures remain exact;
- no unresolved high/medium compatibility defect remains in the claimed
  surface;
- implementation disposition and frozen head are committed;
- no upstream interaction occurred.

## 14. Stop conditions

Stop and record `blocked` or a separate-plan requirement if:

- fixing compatibility requires implementing a substantial missing base method;
- a router-control or settings owner must be added;
- exact historical behavior cannot be determined from retained code/docs;
- public protocol extensions appear necessary;
- unrelated core/frontend work is required;
- external authority changes materially;
- upstream action is requested without explicit authorization.
