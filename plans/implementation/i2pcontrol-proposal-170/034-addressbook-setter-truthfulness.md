# M034 — AddressBook Setter Truthfulness and Runtime Subscription Control

Status: blocked on M033

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Applicable governance and decisions:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/closure/i2pcontrol-proposal-170/030-closure.md`

Repository baseline:

- accepted M033 implementation/closure head, to be recorded before execution

Hard dependency:

- M033 closed, to avoid concurrent edits to `main.rs` and runtime composition

## 1. Bounded objective

Eliminate successful Proposal 170 AddressBook responses for inert subscription
or configuration metadata.

M034 makes `SetSubscriptions` control the active Emissary AddressBook downloader
through one bounded runtime command seam. `SetConfig` accepts only fields that
can be enforced safely by existing Emissary ownership without arbitrary path
control or a general AddressBook redesign; every other field returns a
deterministic unsupported/error result before persistence.

The default expected supported `SetConfig` set is empty unless implementation
inspection demonstrates an existing live owner for a field. This is preferable
to persisting metadata and claiming success. Full Java AddressBook configuration
parity is not authorized by this milestone.

M034 must preserve M028 disabled-mode isolation and M030 full-destination owner
coherence.

## 2. Current defect and why prior evidence missed it

The runtime owner persists `subscriptions` and `configuration` collections.
`runtime_set_subscriptions()` explicitly stores metadata without fetching, and
`runtime_set_configuration()` stores non-operative metadata. The running
`AddressBookManager` retains its startup subscription vector and does not consume
later control-state mutations. Canonical handlers nevertheless return success.

Earlier milestones proved persistence, response shape, owner coherence, and
feature isolation. They did not require an operational consumer for these
setter modes.

M034 adds the missing capability evidence: a successful setter must change the
actual live owner behavior, not only a JSON file.

## 3. Required invariants

1. Success means the active runtime owner accepted and applied the supported
   operation.
2. Inert metadata is never reported as a successful runtime modification.
3. `SetSubscriptions` replaces one bounded source set atomically.
4. A successful active `SetSubscriptions` queues at most one bounded refresh;
   duplicate/coalesced commands cannot create unbounded work.
5. Refresh uses the existing proxy/download/parse/merge path and preserves M030
   full-destination validation and owner publication.
6. Failure to queue or apply the runtime change leaves the prior durable and
   active subscription set.
7. If the AddressBook downloader is unavailable, the request fails explicitly;
   it does not report deferred success.
8. Request-selected file paths from `SetConfig` are rejected.
9. Unsupported `SetConfig` keys are rejected before mutation.
10. No general scheduler, generic configuration bus, bidirectional file sync,
    or second AddressBook authority is introduced.
11. Disabled/runtime-disabled execution does not construct or consult the
    control command seam.
12. Existing entry add/delete/update/list/lookup and Base32/Base64 coherence
    remain unchanged.
13. No tunnel, RouterInfo source, core, frontend, CI/release, or upstream work.

## 4. Scope and file budget

### Primary production scope

- `emissary-cli/src/i2pcontrol/address_book.rs`;
- `emissary-cli/src/i2pcontrol/production.rs`;
- I2PControl AddressBook domain/control traits and tests;
- directly affected server/composition code.

### Permitted original AddressBook seam

`emissary-cli/src/address_book.rs` may add only:

- one bounded typed control channel/handle for replacing subscriptions and
  requesting refresh;
- one live snapshot of the active subscription set;
- cancellation-safe command processing integrated with the existing manager
  task;
- focused tests.

The original module must not parse Proposal 170 request objects or own
administrative response policy.

### Conditional composition scope

`emissary-cli/src/main.rs` may pass the control handle to the production
I2PControl adapter only when the feature is compiled and runtime-enabled.

### Hard exclusions

- arbitrary path setters;
- generic file-layout reconfiguration;
- theme/log settings;
- new HTTP proxy architecture;
- new cross-platform watcher/event bus;
- core, tunnel, RouterInfo, frontend, CI/release, or unrelated changes.

## 5. Target operation model

### 5.1 SetSubscriptions

Canonical request replaces the full bounded list.

Required sequence:

1. validate count, entry length, URL/string syntax required by existing
   downloader, and aggregate size;
2. obtain a command permit/reservation without mutating durable state;
3. send a typed replace-and-refresh command to the active manager;
4. manager swaps the active source set and performs one refresh through the
   existing path;
5. successful refresh/publish acknowledges the command;
6. production adapter persists the same accepted set atomically, or uses a
   single owner transaction that cannot diverge;
7. handler returns success only after the chosen operational success point.

The implementation must choose one exact success point and document it. The
preferred definition is: active source replacement and durable publication are
complete, and refresh has been accepted/started; network download success is not
required because remote availability is not under setter control. If refresh
queueing cannot be guaranteed, return error.

### 5.2 Command coalescing and bounds

Use a bounded channel or latest-generation command slot. Multiple replacements
may coalesce to the newest complete set, but each caller must receive a truthful
result for whether its generation became active or was superseded.

No unbounded queue, detached task per request, or lock held across network I/O.

### 5.3 SetConfig

Build an exact key table from the pinned proposal. For each key classify:

- `operational` — an existing Emissary owner can enforce it without arbitrary
  path control or new architecture;
- `unsupported` — return deterministic operation error;
- `invalid for Emissary security boundary` — return invalid params where the
  input is an unsafe arbitrary path or malformed type.

Do not persist unsupported keys. Do not claim a path was modified when Emissary
has no corresponding path-backed source.

The likely initial disposition is:

- path keys (`subscriptions`, published/router/local/private addressbook,
  `etags`, `last_modified`, `log`) — rejected as request-selected paths;
- `theme` — unsupported/non-runtime;
- `proxy_host`/`proxy_port` — unsupported unless the existing bound proxy owner
  exposes a safe live update seam;
- `update_delay` — unsupported unless an existing scheduler owner exists;
- `should_publish` — unsupported unless existing publication behavior can be
  toggled atomically.

A zero-key operational set is acceptable and must be documented honestly.

## 6. Ordered work packages

### WP1 — Freeze false-success behavior

Add tests showing current successful responses do not affect the active
subscription owner. Add fixtures for unsupported/path-based `SetConfig` keys.

### WP2 — Define the exact field disposition table

Pin every proposal config key, expected JSON type/string representation, runtime
owner, supported state, and error behavior in code and documentation. Avoid a
free-form map with silent retention.

### WP3 — Add the bounded runtime subscription handle

Implement the minimal original-module seam and feature/runtime isolation.
Preserve the existing download retry, modified-time, proxy, parse, merge, and
full-destination behavior.

### WP4 — Wire durable and active subscription state

Ensure one accepted subscription generation is authoritative. Prevent durable
state from succeeding while the active manager retains the previous set.

### WP5 — Correct handler responses

Return success only for operational accepted mutations. Return canonical textual
operation errors or JSON-RPC invalid params according to the existing contract
boundary; do not invent status fields.

### WP6 — Documentation and disposition

Update AddressBook, support, conformance, and security docs. Create:

- `plans/closure/i2pcontrol-proposal-170/034-implementation-disposition.md`.

## 7. Failure, cancellation, restart, and contention semantics

- Validation failure changes neither durable nor active subscriptions.
- Channel unavailable/full/cancelled returns error and leaves prior state.
- Manager cancellation during command processing acknowledges failure or leaves
  a recoverable generation; it does not silently report success.
- Network refresh failure does not roll back an already accepted source set, but
  is recorded separately and does not corrupt the downloader task.
- Restart reloads the last accepted subscription set into the active manager.
- Disabled restart ignores control state as required by M028.
- Concurrent setters serialize or use generation replacement without partial
  lists.
- No owner lock is held across download/network awaits.

## 8. Compatibility and migration

- Existing valid AddressBook entry state remains unchanged.
- Existing persisted inert configuration metadata must be classified. It may be
  ignored, rejected on activation, or removed through a schema-preserving
  cleanup only if prior state is not silently treated as operational.
- Existing subscription metadata becomes active only after successful M034
  migration/reconciliation.
- Standalone compatibility methods may remain aliases, but must share the same
  truthfulness and runtime behavior.
- No public field or response extension.

## 9. Security review requirements

Review and test:

- path keys never control filesystem locations;
- URL/count/size bounds;
- no token/password/destination/path leakage in errors;
- command channel cannot create unbounded refresh tasks;
- failed refresh cannot crash the manager or I2PControl server;
- disabled mode remains isolated;
- M030 destination validation remains active;
- no upstream interaction.

## 10. Focused tests

Required semantics include:

- `set_subscriptions_updates_active_runtime_sources`;
- `set_subscriptions_restart_restores_accepted_sources`;
- `set_subscriptions_queue_failure_preserves_prior_state`;
- `concurrent_subscription_replacements_publish_complete_generation`;
- `set_config_path_keys_are_rejected`;
- `set_config_unsupported_keys_do_not_persist`;
- `inert_metadata_never_returns_success`;
- `runtime_disabled_setter_state_is_not_consulted`;
- `subscription_refresh_preserves_full_destination_owner`.

## 11. Verification commands

```bash
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features address_book
cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings

cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol address_book
cargo test -p emissary-cli --no-default-features --features i2pcontrol subscriptions
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

Use targeted formatting and `git diff --check`. No broad CI, network farm, fuzz,
soak, release, or packaging work.

## 12. Documentation and static guards

Add guards proving:

- every proposal config key has one explicit disposition;
- unsupported/path keys cannot reach generic persistence;
- original AddressBook module does not import JSON-RPC/handler types;
- command seam is feature/runtime gated;
- no second authority or background task per request;
- no core changes.

## 13. Acceptance criteria

M034 may close only when:

- successful `SetSubscriptions` changes the actual active owner;
- unsupported/inert `SetConfig` fields fail truthfully before persistence;
- M028/M030 invariants pass;
- failure/restart/contention behavior is evidenced;
- no high/medium M034 defect remains;
- all outside-I2PControl changes are minimal and justified;
- implementation disposition and frozen head are committed;
- no upstream interaction occurred.

## 14. Stop conditions

Stop and record `blocked` if:

- operational subscription replacement requires a general AddressBook scheduler
  or proxy redesign;
- safe behavior requires arbitrary path control;
- a second authority or unbounded command queue appears necessary;
- M030 owner coherence would be weakened;
- unrelated core/tunnel/frontend work becomes necessary;
- proposal authority changes materially;
- upstream action is requested without explicit authorization.