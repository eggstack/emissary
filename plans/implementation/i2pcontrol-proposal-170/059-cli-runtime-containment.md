# M059 — Original CLI and Runtime Adapter Containment

Status: ready

Planning baseline: to be pinned by accepted M058 closure

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`

Accepted predecessor and budget:

- M058 closure and machine-readable containment ledger;
- exact M059 changed-path budget frozen by M058;
- no `uncertain` ledger entry may be modified under this milestone.

Milestone class: corrective implementation

Applicable authority:

- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`;
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`;
- `plans/implementation/i2pcontrol-proposal-170/037-containment-boundary-reduction.md`;
- accepted M058 ledger and closure.

## 1. Bounded objective

Reduce Proposal-170-derived policy and machinery in original `emissary-cli` runtime modules while preserving the already accepted operational behavior.

M059 is confined to original CLI/runtime paths classified by M058. Its target is to leave those modules with only the minimum functionality they uniquely own:

- feature/configuration parsing required to enable or bind the service;
- startup/shutdown composition;
- neutral runtime lifecycle notifications or capability adapters;
- legacy runtime behavior that exists independently of I2PControl.

Everything else—Proposal 170 method semantics, administrative persistence policy, support classification, aggregation/bounds, JSON-RPC terminology, and control-plane state—belongs under `emissary-cli/src/i2pcontrol/**`.

M059 does not touch `emissary-core`.

## 2. Current evidence and expected target groups

M058 determines the authoritative path budget. At planning time, the expected original CLI/runtime candidates include:

- `emissary-cli/src/address_book.rs`;
- `emissary-cli/src/config.rs`;
- `emissary-cli/src/lib.rs`;
- `emissary-cli/src/logger.rs`;
- `emissary-cli/src/main.rs`;
- `emissary-cli/src/proxy/http/error.rs`;
- `emissary-cli/src/proxy/http/mod.rs`;
- `emissary-cli/src/proxy/http/request.rs`;
- `emissary-cli/src/proxy/socks.rs`;
- `emissary-cli/src/tunnel/client.rs`;
- `emissary-cli/src/tunnel/server.rs`;
- package manifests/lockfile only if dependency removal follows directly from the containment edits.

The final list must come from M058; this seed list is not permission to touch a path omitted from the accepted ledger budget.

The largest containment concern is `address_book.rs`, which historically accumulated administrative overlay/persistence behavior before M037 moved substantial policy into `i2pcontrol`. The proxy/tunnel/logger paths are expected to contain passive service-observation/runtime-registration seams. `main.rs` and `config.rs` are expected to retain some unavoidable feature/composition code.

## 3. Required invariants

1. Exact Proposal 170/base I2PControl wire behavior remains unchanged.
2. Authentication, TLS, throttling, server-secret storage, and control-plane persistence remain inside `i2pcontrol`.
3. AddressBook enabled/disabled behavior, Base32/Base64/full-destination coherence, subscription commit behavior, restart behavior, and generation durability remain unchanged.
4. Supported client/server tunnel creation, start/stop/restart/delete behavior remains unchanged.
5. Unsupported tunnel backends remain resource-free and return the accepted unsupported behavior.
6. ClientServicesInfo observes the same supported proxy/tunnel/service activity as before.
7. Original proxy/tunnel modules do not acquire JSON-RPC/control-plane request types.
8. No original CLI/runtime module becomes a second administrative store or runtime authority.
9. No core path changes are authorized.
10. No new background task, global registry, event bus, persistent store, probe, or dependency is introduced for containment.
11. The accepted RouterInfo 37/1/5 disposition is invariant.
12. No upstream interaction is authorized.

## 4. Scope and changed-path budget

The exact authorized production paths are those tagged `next_milestone = "M059"` in the accepted M058 ledger.

Always authorized within this milestone:

- `emissary-cli/src/i2pcontrol/**` for moving/extracting control-plane policy and adapters;
- focused `emissary-cli/tests/**` required to preserve behavior and enforce containment;
- `docs/i2pcontrol/**` only where ownership documentation must be corrected;
- M059 planning/closure records;
- `emissary-cli/Cargo.toml`, root `Cargo.toml`, and `Cargo.lock` only when a dependency becomes provably unused because of M059 code movement/removal.

Explicitly prohibited:

- `emissary-core/**`;
- unrelated CLI/UI/runtime paths not in the M058 budget;
- `.github/**`;
- release/publishing configuration;
- any unsupported tunnel implementation.

If M059 appears to require a core edit, stop and defer the issue to M060 or a separate corrective plan. Do not expand this milestone.

## 5. Target ownership by area

### 5.1 AddressBook

Original `address_book.rs` should own only ordinary Emissary AddressBook runtime responsibilities and narrow generic seams required by the control overlay.

Prefer under `i2pcontrol/**`:

- administrative DTOs and validation;
- control-state generation/persistence policy;
- legacy control-state import/repair policy;
- subscription/config administrative semantics;
- Proposal 170 error mapping;
- administrative bounds and publication coordination;
- any migration logic that exists solely because of I2PControl control-state files.

Retain in original AddressBook code only when required independently of I2PControl:

- downloader/source parsing used by normal runtime;
- resolution/publication behavior that is already part of ordinary Emissary;
- a narrow optional owner/overlay interface for validated administrative entries or subscription updates;
- no JSON-RPC types or Proposal 170 persistence schema knowledge.

Do not create a second resolver map or durable owner inside I2PControl merely to make the original file smaller.

### 5.2 Configuration and composition

`config.rs` may retain feature-gated configuration keys required to enable/listen/configure I2PControl. It should not contain method inventory, selector semantics, administrative defaults masquerading as router state, or backend policy.

`main.rs` may:

- construct the already-defined I2PControl production composition;
- pass neutral owner/observer handles;
- start/stop the control server with existing router lifecycle;
- handle startup failure according to accepted server behavior.

`main.rs` must not assemble RouterInfo/ClientServicesInfo/TunnelManager semantics itself.

`lib.rs` should expose only what tests/binary composition actually require. Remove public exports introduced solely as convenience if they are unnecessary after the move, provided this does not break a documented public library contract.

### 5.3 Logger

If original logger code was modified solely to provide I2PControl log observation, retain only a generic passive sink/snapshot hook that is independently reasonable and bounded. Move log-history limits, JSON serialization, filtering, control-plane generation semantics, and RPC mapping into `i2pcontrol`.

If existing logger facilities can already provide the required observation without the fork delta, revert the I2PControl-specific logger modification instead.

No unbounded log buffer or second logging pipeline.

### 5.4 Proxy HTTP/SOCKS

Original proxy modules may publish only sanitized lifecycle facts required for ClientServicesInfo or runtime registry truthfulness.

Prefer:

- one optional lightweight observer/registry handle at service construction;
- owner-local active/inactive transitions;
- stable non-sensitive identifiers/configuration facts already known by the service.

Do not retain:

- ClientServicesInfo response DTOs;
- service type/status wire mapping;
- JSON-RPC types;
- aggregate sorting/bounds;
- second service registries maintained independently of actual runtime lifecycle.

If several modified proxy files merely thread the same observer through internal request/error helpers, consolidate at the highest owner that can truthfully observe service lifecycle and revert lower-level changes.

### 5.5 Existing client/server tunnel runtime modules

The original client/server tunnel implementations may expose only neutral lifecycle/configuration adapters needed by the I2PControl backend to start/stop the existing runtime.

I2PControl owns:

- definition IDs/revisions;
- persisted administrative definitions;
- state-machine policy;
- JSON response/status mapping;
- unsupported type classification;
- restart/delete atomicity and administrative error translation.

Original tunnel modules should not know `TunnelManager`, Proposal 170 method names, or persistence formats.

Do not alter the actual tunnel data path or session behavior.

## 6. Ordered work packages

### WP1 — Freeze M058 path budget and behavior tests

Read the accepted M058 ledger and closure. Copy the exact authorized M059 path list into the implementation disposition/working notes before editing.

Run the focused pre-change tests needed by each path. Record any pre-existing failure; do not normalize it away.

### WP2 — Remove policy leakage before structural cleanup

For each `policy_leak = true` original CLI path:

1. identify the smallest policy block that belongs under `i2pcontrol`;
2. move it without changing its storage/wire semantics;
3. leave a neutral interface only where the original owner must perform the runtime action;
4. run focused tests immediately.

Do not combine this with broad renaming or style refactors.

### WP3 — Consolidate passive CLI/runtime adapters

For paths marked `candidate-consolidate`, reduce observer/registry threading where a higher-level original runtime owner can emit the same authoritative lifecycle event.

The proof requirement is semantic equivalence, not merely compilation. The old and new observers must produce the same accepted fixture/lifecycle result.

### WP4 — Revert candidate original-CLI paths

For each M058 `candidate-revert` path, restore upstream behavior and route the accepted control-plane consumer through an existing higher-level seam where applicable.

If the revert changes observable supported API behavior, restore the path and record it as required rather than weakening semantics.

### WP5 — Dependency/export cleanup

After code movement:

- remove imports, public exports, feature dependency edges, or third-party dependencies that have no remaining consumer;
- do not chase binary-size or dependency cleanup beyond what M059 directly makes unused;
- preserve feature-off compilation.

### WP6 — Static containment regressions

Add/update focused guards proving at minimum:

- original CLI/runtime budget paths do not contain JSON-RPC request/response ownership;
- AddressBook original module does not own the administrative store schema/policy;
- proxy/tunnel runtime modules expose no control-plane DTOs;
- no core file changed;
- unsupported backends remain resource-free.

Prefer extending existing containment tests if that remains clear; do not build a generic repository policy engine.

### WP7 — Documentation and closure

Update ownership documentation only where behavior/module ownership changed.

Create `plans/closure/i2pcontrol-proposal-170/059-closure.md` with:

- pinned baseline and M058 budget;
- before/after path/hunk ownership table;
- exact production paths changed;
- paths reverted to upstream;
- paths retained with owner rationale;
- focused verification outcomes;
- dependency/export changes, if any;
- explicit confirmation that `emissary-core/**` did not change;
- internal-only/no-upstream attestation.

Only accepted M059 closure makes M060 ready.

## 7. Failure, cancellation, restart, and contention semantics

### AddressBook

- Failed administrative publication leaves the prior accepted live/durable generation.
- Disabled I2PControl mode must retain ordinary AddressBook behavior with no administrative overlay authority.
- Moving code must not change subscription download task cancellation or restart ownership.
- No lock is introduced across network I/O, filesystem enumeration, or `.await`.

### Proxy/tunnel observers

- Observer failure must not stop, start, cancel, or otherwise alter the underlying service.
- Service lifecycle is authoritative; observation is best-effort but the I2PControl aggregate must preserve its existing incomplete/fail-closed behavior where required.
- No observer callback may block the service poll/data path on network or filesystem work.

### Server composition

- Control-server startup/shutdown semantics remain as accepted by M040/M043.
- A control-server failure must not silently change router lifecycle semantics beyond the already accepted behavior.

Restart: all persisted control state and runtime rehydration behavior remains identical to the pre-M059 baseline.

## 8. Compatibility and migration

No wire or persistence migration is authorized.

Existing AddressBook control state, tunnel definitions, server secret/token state, and configuration files must remain readable with identical semantics.

Any moved type whose serialized form is persisted must preserve exact serde field names/defaults/version behavior or remain represented by the existing `i2pcontrol` domain type.

Do not rename public CLI flags/config keys as part of containment.

## 9. Security and performance review

Required review:

- no secret/private destination/session material newly crosses adapters;
- no mutable runtime handle is exposed where a bounded command/capability interface suffices;
- no new unbounded queue/map/log buffer;
- observer absent-path overhead is no worse than baseline and should be effectively a no-op;
- no extra clone of large destination/router/session objects solely for control observation;
- feature-off code does not allocate I2PControl state;
- moved AddressBook persistence retains path/permissions/atomic replacement rules;
- no new dependencies unless directly required and explicitly justified.

## 10. Focused tests and verification

Minimum local commands, adjusted only to actual package feature names at execution time:

```bash
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m037_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test client_services_integration
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m033_tunnel_lifecycle
cargo test -p emissary-cli --no-default-features --features i2pcontrol address_book
cargo test -p emissary-cli --no-default-features address_book
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

Also run targeted tests named by each M058 ledger entry.

Changed-path proof:

```bash
git diff --name-only <M059_BASE>..HEAD
```

The result must contain no `emissary-core/**` path.

No hosted CI, coverage, fuzz, soak, or release verification is required.

## 11. Documentation and static guards

Update `docs/i2pcontrol/inspection-architecture.md`, `security.md`, `address-book.md`, `client-services.md`, or `tunnel-backends.md` only if the ownership description became stale. Do not rewrite unrelated user documentation.

Static guards should verify ownership by source dependency/term constraints rather than brittle line numbers.

## 12. Acceptance criteria

M059 may close only when all are true:

1. Every changed original CLI/runtime path is within the accepted M058 M059 budget.
2. No `emissary-core/**` production path changed.
3. Proposal 170/JSON-RPC administrative and wire policy is owned under `emissary-cli/src/i2pcontrol/**`.
4. Original `address_book.rs` retains only ordinary runtime ownership and the smallest neutral control overlay seam required for accepted behavior; every retained I2PControl-related block has direct owner justification.
5. Proxy HTTP/SOCKS original modules contain only minimal neutral lifecycle observation required for accepted ClientServicesInfo behavior; avoidable deep observer threading is removed.
6. Existing client/server tunnel modules contain only neutral runtime capabilities/lifecycle hooks and no TunnelManager wire/persistence policy.
7. Logger/config/main/lib changes are reduced to minimum required logging hook, feature/configuration, composition, and necessary exports respectively.
8. Every M058 `candidate-revert` in the M059 budget is either reverted or closure explains with regression evidence why it must be reclassified as required.
9. No new duplicate runtime/admin authority, background task, event framework, or dependency was introduced.
10. No persisted format, config key, public wire behavior, or supported tunnel lifecycle changed.
11. Feature-off and feature-on focused verification passes, except explicitly recorded pre-existing failures that do not result from M059.
12. Accepted RouterInfo source matrix remains 37/1/5.
13. Unsupported tunnel types remain unsupported/resource-free.
14. Closure records before/after retained-path rationale and exact changed-path proof.
15. No upstream interaction occurred.

## 13. Stop conditions

Stop and record a blocker rather than expanding M059 if:

- a required change falls outside the M058 path budget;
- a containment move requires `emissary-core` changes;
- moving AddressBook logic would create a second resolver/publication authority or require a schema migration;
- a proxy/tunnel observer cannot be moved upward without losing authoritative lifecycle ordering;
- a public API/config/wire change appears necessary;
- a current supported behavior is found to be semantically incorrect rather than merely poorly located;
- a new framework/general event bus seems necessary;
- an external Proposal 170 revision changes the contract materially;
- any upstream write/review/submission action is proposed.

## 14. Expected closure disposition

Successful M059 closure should state that original CLI/runtime containment is corrected, no core path changed, supported behavior is preserved, and M060 may proceed against a smaller/explicit core-only containment budget.
