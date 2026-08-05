# M039 — Proposal 170 Operational Final-Head Reclosure

Status: partial Proposal 170 support

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/039-operational-reclosure.md`

Reviewed final head: `8de9734` — `docs: close I2PControl live runtime validation`

Accepted M038 implementation/evidence head: `a5864d2` — `test: validate I2PControl live runtime interoperability`

M030 baseline: `29b42f29fdd98914ef95d44f80f9353175019ee0`

Review date: 2026-08-05

## 1. Final disposition

M039 is formally closed as `partial Proposal 170 support`. The independent
review of the complete M031–M038 workstream found that every implemented and
claimed dimension is exact, operational, bounded, and evidenced. The generic
control-plane `client` and `server` backends are real production backends, and
the other ten declared tunnel families remain exhaustive, explicit,
resource-free unsupported backends.

The partial disposition is required by the retained roadmap boundary: 26 of
the 43 Proposal 170 RouterInfo additions remain unavailable because Emissary
has no bounded canonical owner for those sources, and ten tunnel data planes
remain intentionally unsupported. The live run also records two qualified
environment/composition limitations: this local configuration had no reseeded
peer set for client/server traffic formation, and it had no composed HTTP
downloader for positive subscription refresh. Neither limitation was replaced
with fabricated success.

No corrective pass is required. No high- or medium-severity correctness,
security, compatibility, ownership, containment, or evidence defect remains in
an implemented dimension.

## 2. External authority and dependency closure

The official Proposal 170 page was rechecked read-only on 2026-08-05:

- title: `I2PControl Expansion`;
- status: `Open`;
- created: `2026-05-20`;
- last updated: `2026-05-20`;
- page: <https://i2p.net/en/proposals/170-i2pcontrol-expansion/>.

The pinned revision is unchanged, so M039 is not blocked pending a contract
rebase. M030–M038, including M038A and M038B, were reviewed through their
accepted closure/disposition records. M020–M030 evidence remains valid and is
retained except where later records explicitly supersede its final status
language.

There are no registered successor implementation plans after M039. The
roadmap's deferred work (unavailable RouterInfo owners and unsupported tunnel
families) is not dependency-ready work and remains unregistered. Therefore no
future plan can be unblocked by this closure; no successor status was changed
to `ready`.

## 3. Requirement-to-evidence matrix

| Requirement | Exact evidence | Result |
|---|---|---|
| Authentication and token behavior | `emissary-cli/src/i2pcontrol/auth.rs`; `server.rs`; `golden_fixtures`, `adversarial`, and live-runtime tests | Pass: API/password authentication, opaque bounded tokens, protected `params.Token`, restart invalidation, and standard errors are retained. |
| Failed-login throttling | `auth::AuthThrottle`; `auth` unit tests; `adversarial` tests; feature clippy | Pass: peer-keyed bounded table, monotonic bounded delay, eviction, and successful reset. |
| Direct/base compatibility | `rpc.rs` inventories and mode dispatcher; `m027_literal_fixtures`; `golden_fixtures`; `conformance_manifest` | Pass: direct presence semantics and historical nested/base semantics are distinct; exact overlaps are table-driven; unsupported base methods fail explicitly. |
| RouterInfo method family | `router_info_handler.rs`; `rpc.rs::router_info_keys`; `router_info_truthfulness`; `production_adapter`; literal fixtures | Pass: exact selectors, types, casing, preflight, bounded assembly, and sanitized unavailable errors. |
| AddressBook method family | `address_book.rs`; `address_book_runtime.rs`; `golden_fixtures`; `production_adapter`; live runtime | Pass/qualified: add, replace, delete, list, subscriptions, and explicit unsupported configuration behavior are truthful and durable. |
| TunnelManager method family | `tunnel_manager.rs`; domain/handler tests; `m033_tunnel_lifecycle`; `production_adapter`; live runtime | Pass: exact seven actions, twelve types, validation, result shapes, `All`, CRUD, and lifecycle errors. |
| ClientServicesInfo method family | `client_services.rs`; `client_services_integration`; `client_services_live`; live runtime | Pass: requested-only selectors use actual owners, bounded SAM observation is recoverable, and BOB is exactly `false`. |
| Generic client backend | `backends/client.rs`; `tunnel/client.rs`; `production.rs`; `m033_tunnel_lifecycle`; live runtime | Pass: control-plane-owned supervisor reuses the existing data plane with bounded start/stop/restart and generation fencing. Traffic formation remains unclaimed without a reseeded peer set. |
| Generic server backend and secret identity | `backends/server.rs`; `server_secret_store.rs`; `tunnel/server.rs`; `production_composition`; live runtime | Pass/qualified: backend-owned fixed-path identity, redacted private material, recovery, and lifecycle are real; public destination/traffic formation is not claimed without peers. |
| Unsupported tunnel families | `backends/registry.rs`; `backends/unsupported.rs`; static guards; `production_adapter`; live runtime | Pass: all twelve types register exactly once; the ten unsupported families return explicit not-implemented status and allocate no listener, destination, session, task, or traffic path. |
| Startup-managed ownership | `production.rs`; `tunnel_manager.rs`; `m033_tunnel_lifecycle`; live runtime | Pass: startup definitions are externally owned, observable, and reject administrative mutation/adoption. |
| StartOnLoad | `tunnel_manager.rs`; `production.rs`; `m033_tunnel_lifecycle`; live restart evidence | Pass: only eligible control-plane generic client/server definitions auto-start after durable load; unsupported and startup-managed definitions do not. |
| Restart/delete/edit/rename/All | `tunnel_manager.rs`; `m033_tunnel_lifecycle`; `persistence_concurrency`; live runtime | Pass: exact-name serialization, stop-before-start, delete coordination, running-rename rejection, bounded `All`, and durable edit/rename behavior. |
| Failure recovery | `backends/client.rs`, `backends/server.rs`, `tunnel_manager.rs`; lifecycle and live-runtime tests | Pass: bind/SAM failure records stopped/failed state, releases the generation/name, preserves prior durable state, and recovers without router restart or store deletion. |
| AddressBook entry owner coherence | `address_book_runtime.rs`; `address_book.rs`; `router_info_handler.rs`; M030 closure; owner-coherence tests | Pass: administrative, RouterInfo, Base32, and Base64 views share one enabled-mode full-destination owner; disabled/default modes do not consult control state. |
| RouterInfo 43-field source classification | `rpc.rs::router_info_keys::PROPOSAL_170_CONTRACT`; `docs/i2pcontrol/router-info-source-map.md`; `router_info_truthfulness`; conformance fixtures | Pass: 16 available, 1 protocol-permitted neutral, and 26 unavailable; unavailable values are never fabricated. |
| Persistence/recovery/durability | `stores/publication.rs`; generation/address-book/server-secret stores; `persistence_concurrency`; adversarial tests; security docs | Pass with platform qualification: synced publication, atomic rotation, prior-generation fallback, stale-temp handling, confinement, and directory-sync qualification are implemented. |
| Feature isolation | `emissary-cli/src/address_book.rs`; feature guards; no-feature CLI suite; M028/M029 retained tests | Pass: no-feature and runtime-disabled execution does not initialize, read, write, or consult Proposal 170 control state. |
| Security bounds and secret handling | `server.rs`, `auth.rs`, `stores/publication.rs`, `server_secret_store.rs`; adversarial/security tests; feature clippy | Pass: request/connection/concurrency bounds, constant-time password primitive, throttling, fixed paths, restrictive permissions, redaction, and sanitized errors. |
| Containment boundary | M037 changed-path guard; `tests/m037_containment.rs`; M037 closure; `git diff --name-status` from M030 baseline | Pass: Proposal 170 policy is inside `emissary-cli/src/i2pcontrol/**`; external production changes are narrow runtime/composition hooks, including the behavior-preserving passive core seam. |
| Live-runtime validation | `emissary-cli/tests/i2pcontrol_live_runtime.rs`; production TLS/auth stack; exact command in §5 | Pass: real feature-enabled child process covered authentication, notifications, AddressBook, RouterInfo, ClientServicesInfo, TunnelManager, failure/recovery, restart, ownership, unsupported paths, and cleanup. |
| No-upstream compliance | repository log/status; planning governance §11; this record's attestation | Pass: external sources were read-only; no upstream repository/channel or review workflow was mutated. |

## 4. Review dimensions and invariants

### Wire and compatibility

The canonical Proposal 170 method names, selector names, actions, tunnel types,
field casing, JSON types, presence semantics, response shapes, and error
channels remain represented by the machine-readable inventories and literal
fixtures. Direct Proposal 170 requests are not conflated with the historical
nested/base compatibility form. No new alias, method, status, or tunnel type
was added by M031–M038.

### Runtime and lifecycle

Production composition registers exactly one backend for each of the twelve
types. Only control-plane-created generic `client` and `server` definitions
receive real supervisors. Startup-managed definitions remain external and
read-only. Unsupported backends are resource-free. Per-name generations fence
stale completions; stop is awaited before restart; failures remove stale task
state and leave definitions stopped. No task failure requires a router restart
or store deletion to recover.

### Persistence and recovery

Definitions, AddressBook state, and server identity use owner-controlled,
confined paths and recoverable current/prior or generation publication. Live
snapshots update only after the selected publication point. Cancellation and
directory-sync failures preserve the prior live state. Existing data files and
schema versions remain readable; no public migration or router configuration
format change was introduced. Power-loss durability is qualified by platform
directory-sync capability as documented in `docs/i2pcontrol/security.md`.

### Security

Authentication precedes protected dispatch. Password comparison uses the
reviewed `subtle` primitive; failed authentication is bounded per peer. Request
bodies, concurrent requests, TLS connections, collections, and runtime tasks
are bounded. Secrets are redacted and private server material never crosses a
request-selected path or response. Non-loopback binding is explicit and
warned. Errors do not expose credentials, private keys, arbitrary paths, or
backtraces.

### Containment

The M030-to-final production comparison was classified as follows:

- `emissary-cli/src/i2pcontrol/**`: Proposal 170 wire, state, handlers,
  backends, security, composition, and tests;
- `emissary-cli/src/tunnel/client.rs` and `tunnel/server.rs`: narrow
  cancellation-aware single-instance runtime adapters for existing Yosemite
  data planes;
- `emissary-cli/src/address_book.rs`: one typed active-subscription/runtime
  owner seam preserving legacy behavior when inactive;
- `emissary-cli/src/main.rs`, `src/lib.rs`, and Cargo manifests/lockfile:
  feature and production composition wiring only;
- `emissary-core/src/lib.rs`, `router/mod.rs`, `sam/mod.rs`, and
  `sam/session.rs`: the M037 minimal passive SAM observation seam, with no
  router algorithm, protocol, or data-plane policy;
- `emissary-cli/tests/**` and `plans/**`/`docs/**`: evidence, guards, and
  records.

No frontend, CI, release, arbitrary path, broad core refactor, or upstream
artifact was added.

## 5. Verification outcomes

All required commands were run against the reviewed clean head. Outcomes:

| Command | Outcome |
|---|---|
| `cargo check -p emissary-cli --no-default-features` | pass |
| `cargo test -p emissary-cli --no-default-features` | pass — 54 tests |
| `cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings` | pass |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest` | pass — 58 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test golden_fixtures` | pass — 44 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m027_literal_fixtures` | pass — 7 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter` | pass — 20 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition` | pass — 8 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture` | pass — 1 live test |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | pass — 1,325 tests |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass |
| `cargo check -p emissary-core` | pass |
| `cargo test -p emissary-core sam` | pass — 149 tests, 906 filtered |
| `cargo clippy -p emissary-core --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | known pre-existing failure: stable rustfmt cannot honor this repository's nightly-only options and reports unrelated baseline diffs; no formatter output was retained |
| `git diff --check` | pass |

The live test used real production composition, loopback, temporary state, and
process-local credentials. It did not use an in-process fake in place of the
production server.

## 6. Failure, compatibility, migration, and residual risk

- Failure recovery is bounded and local. Bind/SAM/publication failure leaves
  prior durable state coherent; no router restart or store deletion is
  required.
- Cancellation and stale-generation completion are fenced by per-name
  generation checks. Locks are not held across network I/O, sleeps,
  cancellation, joins, or publication calls.
- Existing configuration and current/backup/generation data formats remain
  readable. No public migration was introduced; server identity uses the
  internal fixed `server-destinations/` store.
- `StartOnLoad` is active only for eligible control-plane generic client/server
  definitions and is not used to adopt startup-managed or unsupported entries.
- Unsupported base methods and the ten unsupported tunnel families return
  explicit errors/status and do not allocate runtime resources.
- Passwords, tokens, private destinations, private keys, and request-selected
  paths are not emitted or accepted across the API boundary. Unix permissions
  and directory durability retain the platform qualifications documented in
  the security record.
- Residual risk is low for the implemented dimensions. The retained high-level
  capability gap is intentional roadmap scope (ten unavailable tunnel data
  planes and 26 unavailable RouterInfo sources), not a defect in a claimed
  implementation. The local no-peer and no-downloader limitations affect only
  positive formation/refresh evidence.

## 7. Documentation and planning disposition

Updated in this closure:

- `plans/registry.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/implementation/i2pcontrol-proposal-170/039-operational-reclosure.md`;
- `docs/i2pcontrol/README.md`;
- `docs/i2pcontrol/proposal-170-support.md`;
- `docs/i2pcontrol/proposal-170-conformance.md`;
- `docs/i2pcontrol/security.md`;
- `docs/i2pcontrol/tunnel-backends.md`;
- `docs/i2pcontrol/tunnel-manager.md`.

The subsystem and implementation plan now record M039 as closed with the
overall `partial Proposal 170 support` disposition. No future plan was
unblocked because no successor is registered and deferred capability work has
no accepted implementation handoff.

## 8. Internal-only attestation

External Proposal 170 and reference material were accessed read-only for
contract verification. No upstream repository, issue, pull request, review,
merge/adoption/submission channel, or maintainer channel was mutated. No
upstream review, merge, adoption, or submission was requested. No contribution
artifact was prepared under M039. The only push authorized by the maintainer
directive is publication of this internal `eggstack/emissary` branch.

**Disposition: `partial Proposal 170 support`; M039 closed.**
