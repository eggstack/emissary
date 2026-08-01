# I2PControl Proposal 170 Milestone M027 — Final Internal Reclosure

Status: partial Proposal 170 support

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/027-proposal-170-conformance-and-reclosure.md`

Implementation disposition:

- `plans/closure/i2pcontrol-proposal-170/027-implementation-disposition.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Invalidated historical closure retained:

- `plans/closure/i2pcontrol-proposal-170/019a-closure-invalidation.md`

## 1. Executive disposition

M027 is closed as `partial Proposal 170 support`. The review found no
unresolved high/medium defect in authentication, JSON-RPC execution, canonical
wire behavior, persistence atomicity, source truthfulness, secret handling, or
scope. The exact supported dimensions are closed internally with evidence.

The result is not unqualified Proposal 170 completion. Twenty-six of the
pinned 43 RouterInfo additions remain truthfully unavailable because M026 found
no bounded authoritative owner. Missing tunnel data planes remain explicit
runtime-unsupported stubs under ADR-0001. A protocol-permitted neutral value is
used only for clock skew when no peer estimate exists.

## 2. Independent review and pinned external evidence

The reviewer was the Codex internal closure reviewer, distinct from the
`wr3n-ai` author of the M026 implementation and closure evidence. The reviewer
read the current production code, tests, source map, M020–M026 dispositions,
and the pinned external pages independently.

Pinned external facts, read-only:

- Proposal 170 is titled `I2PControl Expansion`, status `Open`, author Nick2k4,
  created `2026-05-20`, and last updated `2026-05-20`:
  <https://i2p.net/en/proposals/170-i2pcontrol-expansion/>.
- Its RouterInfo addition set includes the exact names/types represented in the
  43-row internal matrix, including the four address-book list selectors and
  the subscription/config objects.
- It defines four address-book identities, seven lowercase TunnelManager
  actions, twelve tunnel types, six ClientServicesInfo selectors, and the
  canonical AddressBook/TunnelManager response examples.
- The existing I2PControl documentation specifies JSON-RPC 2.0, numeric API
  authentication responses, `params.Token`, and error codes `-32001` through
  `-32006`: <https://i2p.net/en/docs/api/i2pcontrol/>.

No upstream or third-party write was made or requested.

## 3. Corrected implementation and fixture head

The corrected implementation baseline reviewed by M027 is the M020–M026 head
`33ebeab`. M027 adds no production implementation. Its evidence change is:

- `emissary-cli/tests/m027_literal_fixtures.rs` — canonical literal fixture
  corpus and literal 43-selector/type manifest.

Compatibility/parser fixtures remain separately named in
`emissary-cli/tests/golden_fixtures.rs` and are not counted as canonical
fixtures. The final evidence and planning commit is recorded in the git
history alongside this record.

## 4. Requirement-to-evidence matrix

`wire`, `source`, `runtime`, `persistence`, and `evidence` in the disposition
column are independent dimensions. A `partial` row is not counted as a full
operational implementation.

| Requirement | Source specification / ADR / roadmap | Production file/function | Focused test or fixture | Command/outcome | Dimension | Residual limitation | Disposition |
|---|---|---|---|---|---|---|---|
| Authenticate uses `API` and `Password`, returns numeric `API` and `Token` | Existing I2PControl; P170 auth examples | `server.rs::handle_authenticate` | `authenticate_uses_standard_params_and_numeric_api`; M027 literal auth fixture | feature test / pass | wire, evidence | token memory is intentionally non-persistent | pass |
| Missing/invalid password and API version errors are exact | Existing I2PControl error table | `server.rs::handle_authenticate` | `authenticate_distinguishes_password_and_api_errors`; M027 error fixture | feature test / pass | wire, evidence | none | pass |
| Protected calls require `params.Token`; header is compatibility-only and conflicts fail closed | Existing I2PControl; M020 | `server.rs::authenticate_protected_request` | protected auth/token tests | feature test / pass | wire, security | header path retained for compatibility | pass |
| Notifications execute and suppress responses; explicit null ID is preserved | JSON-RPC 2.0; M020 | `server.rs::handle_jsonrpc` | notification test; M027 ID matrix | feature test / pass | wire, evidence | HTTP 204 is the local transport behavior | pass |
| Request IDs preserve strings/numbers/null and reject invalid JSON values | JSON-RPC 2.0; M020 | `rpc.rs::parse_request` | `parse_request_rejects_invalid_ids_without_coercion`; M027 ID matrix | feature test / pass | wire, evidence | none | pass |
| Direct base RouterInfo compatibility survives token removal | Existing I2PControl; M020 | `router_info_handler.rs::handle_router_info` | protected base RouterInfo test | feature test / pass | wire, runtime, evidence | base inventory remains separate | pass |
| AddressBook add/replace/delete uses four exact books and presence semantics | P170 AddressBook section | `address_book.rs::handle_canonical_address_book` | canonical AddressBook handler tests; M027 book fixtures | feature test / pass | wire, source, runtime, persistence, evidence | metadata paths are null because no path-backed owner exists | pass |
| AddressBook subscriptions/config use exact result envelope | P170 AddressBook section; M022 | `address_book.rs::handle_canonical_address_book` | canonical shape tests; M027 metadata fixtures | feature test / pass | wire, source, persistence, evidence | `path: null` is truthful | pass |
| AddressBook destination/hostname validation is complete and fail closed | P170 AddressBook; ADR-0001 | `address_book.rs::validate_hostname`, `validate_destination` | invalid hostname/destination tests | feature test / pass | wire, security | no private destination is exposed | pass |
| AddressBook runtime and durable authority are coherent across restart/failure | M022 roadmap exit | `address_book.rs`, runtime AddressBook owner, stores | M022 closure restart/atomicity evidence | prior closure plus feature suite / pass | source, runtime, persistence, evidence | existing runtime precedence is unchanged | pass |
| TunnelManager has seven lowercase actions | P170 TunnelManager section | `tunnel_manager.rs::handle_tunnel_manager` | canonical action tests; M027 action fixtures | feature test / pass | wire, evidence | capitalized aliases are compatibility-only | pass |
| TunnelManager has twelve exact tunnel types | P170 TunnelManager section | `rpc.rs::tunnel_types`, backend registry | registry and all-type tests; M027 type fixtures | feature test / pass | wire, evidence | ten data planes remain unsupported | pass |
| Action-specific parameters, options, ranges, and enums validate | P170 TunnelManager section; M021 | `tunnel_manager.rs::validate_canonical_request` | focused validation tests | feature test / pass | wire, security, evidence | unsupported options do not create a runtime | pass |
| TunnelManager create/edit/rename/delete publishes one generation atomically | ADR-0001; M021 | `stores/generation_store.rs`, `stores/tunnel_store.rs` | atomicity/rename/failure tests | feature suite / pass | persistence, evidence | none within scope | pass |
| TunnelManager `get` has exact nested `info/rawConfig` shape | P170 `get` example | `tunnel_manager.rs::tunnel_definition_to_get_result` | `handler_get_found`; M027 literal get fixture | feature test / pass | wire, evidence | destination values are only emitted when owned | pass |
| Secrets/private keys are rejected, stored once, and not serialized/logged | P170 options; ADR-0001 | tunnel parser/store/serializer | adversarial/static/security tests | feature suite / pass | security, persistence, evidence | future backend secret needs remain bounded | pass |
| Unsupported tunnel CRUD/lifecycle paths are explicit and resource-free | ADR-0001 | `backends/unsupported.rs`, production control plane | unsupported backend tests; all-type fixture | feature suite / pass | wire, runtime, security, evidence | missing data planes remain unsupported | pass |
| Startup-managed ownership, collision, restart, and rename failure are truthful | M023 roadmap exit | `production.rs`, composition seams | `m023_startup_inventory`, production composition tests | feature suite / pass | source, runtime, persistence, evidence | named lifecycle adapter remains unavailable | pass |
| ClientServicesInfo selects exactly six direct keys by presence | P170 ClientServicesInfo section | `client_services.rs::handle_client_services_info` | selector tests; M027 six-key fixture | feature test / pass | wire, evidence | nested Selector is compatibility-only | pass |
| HTTP/SOCKS state clears on listener exit | M023 | service registry/proxy exit seam | proxy lifecycle tests | feature suite / pass | source, runtime, evidence | no new service ownership introduced | pass |
| I2PTunnel inventory and addresses use actual provenance | P170 ClientServicesInfo; M023 | `client_services.rs::resolve_i2ptunnel_live` | startup inventory/address tests | feature suite / pass | source, runtime, evidence | only existing startup/control-plane inventory is exposed | pass |
| SAM is bounded, incomplete on overflow, and recovers | M024 roadmap exit | `emissary-core/src/sam/**` observation seam | SAM observation/recovery tests | core test / pass | source, runtime, evidence | no live-network activation harness | pass |
| I2CP reports actual listener state and BOB is exactly false | P170 ClientServicesInfo | `client_services.rs` | I2CP/BOB tests; M027 response fixture | feature test / pass | wire, source, runtime, evidence | BOB is intentionally unavailable | pass |
| RouterInfo has exactly 43 additions with exact JSON types | P170 RouterInfo section; M025 | `rpc.rs::PROPOSAL_170_CONTRACT` | literal manifest; `conformance_manifest` | feature tests / pass | wire, evidence | base/compatibility keys are separate | pass |
| RouterInfo selects by direct presence and returns requested keys only | P170 examples; M025 | `router_info_handler.rs::handle_router_info` | direct presence and requested-only tests; M027 mixed fixture | feature tests / pass | wire, evidence | unavailable mixed requests return no partial result | pass |
| Available RouterInfo sources have bounded owners and serialized response limits | M025/M026 source map | `router_info_handler.rs`, production adapters | available source/bound/oversized tests | feature suite / pass | source, runtime, evidence | counts are 16 available, 1 neutral, 26 unavailable | pass |
| Clock skew uses only protocol-permitted neutral null | P170 RouterInfo type; M025 | RouterInfo source matrix/serializer | neutral fixture and source-map row | feature tests / pass | wire, source, evidence | no peer estimate owner | pass |
| Logs/log-clear/transit/rate/address-book/I2PTunnel semantics are exact | P170 RouterInfo section; M025 | `router_info_handler.rs` | literal/semantic RouterInfo tests | feature suite / pass | wire, source, runtime, evidence | rolling 15s source unavailable | pass |
| Unavailable RouterInfo selectors fail sanitized and no fabricated default | ADR-0001; M026 | RouterInfo preflight/inspection errors | exhaustive 26-field unavailable test | feature suite / pass | source, evidence, security | exact unavailable set listed below | pass |
| Source failures and oversized responses return no partial result | M025/M026 roadmap exits | RouterInfo assembly and bounds | no-partial/source-failure/serialized-bound tests | feature suite / pass | source, evidence, security | none | pass |
| Concurrency observes coherent before/after generations and restart fallback | ADR-0001; M021/M022 | generation stores and owner handles | store contention/restart/corruption tests | package tests / pass | persistence, evidence | response-lost-after-commit is documented operationally | pass |
| Sensitive material is absent from representative serialized responses/logs | ADR-0001 security section | serializers, errors, tracing | `adversarial`, `static_guards`, security tests | package tests / pass | security, evidence | no full private destinations outside required fields | pass |

## 5. Final RouterInfo source/runtime table

### Available — source and runtime supported (16)

`i2p.router.id`, `i2p.router.info`, `i2p.router.logs`,
`i2p.router.logs.clear`, `i2p.router.net.total.received.bytes`,
`i2p.router.net.total.sent.bytes`, `i2p.router.net.total.transit.bytes`,
`i2p.router.net.tunnels.shareratio`, `i2p.router.net.tunnels.i2ptunnel`,
`i2p.router.net.tunnels.totalsuccessrate`,
`i2p.router.addressbook.private.list`, `i2p.router.addressbook.local.list`,
`i2p.router.addressbook.router.list`, `i2p.router.addressbook.published.list`,
`i2p.router.addressbook.subscriptions`, and
`i2p.router.addressbook.config`.

These rows have bounded authoritative owners recorded in
`docs/i2pcontrol/router-info-source-map.md` and are the only rows counted as
available.

### Protocol-permitted neutral — source unavailable but exact neutral allowed (1)

`i2p.router.clockskew` returns `null` when no peer estimate exists. This is a
protocol-permitted neutral value, not a fabricated integer or a claim of a
clock-skew source.

### Unavailable — exact list (26)

The following selectors fail with sanitized internal errors before assembly:

- `i2p.router.news`
- `i2p.router.net.bw.transit.15s`
- `i2p.router.net.tunnels.participating.info`
- `i2p.router.net.tunnels.exploratory.inbound`
- `i2p.router.net.tunnels.exploratory.outbound`
- `i2p.router.net.tunnels.exploratory.info.list`
- `i2p.router.net.tunnels.client.inbound`
- `i2p.router.net.tunnels.client.outbound`
- `i2p.router.net.tunnels.client.info.list`
- `i2p.router.net.status.v6`
- `i2p.router.net.error`
- `i2p.router.net.error.v6`
- `i2p.router.net.testing`
- `i2p.router.net.testing.v6`
- `i2p.router.net.tunnels.successrate`
- `i2p.router.net.tunnels.queue`
- `i2p.router.net.tunnels.tbmqueue`
- `i2p.router.netdb.peers`
- `i2p.router.netdb.activepeers.info`
- `i2p.router.netdb.ntcp.limit`
- `i2p.router.netdb.ssu.limit`
- `i2p.router.netdb.bannedpeers`
- `i2p.router.netdb.activepeers.list`
- `i2p.router.netdb.peers.list`
- `i2p.router.netdb.peers.info`
- `i2p.router.netdb.activepeers.stats`

No unavailable row is represented by zero, false, empty, or partial success.

## 6. Failure, restart, cancellation, contention, and source review

- Authentication rejects missing/unknown/conflicting credentials before
  protected handler work. Notification requests still execute validation and
  side effects before returning 204.
- Tunnel create/edit/rename/delete and AddressBook mutations use the existing
  serialized owners and complete-generation publication. Failed publication
  leaves the prior state; restart loads the same accepted generation or falls
  back to the prior complete generation.
- Startup-owned names collide fail closed. Proxy exit uses generation fencing
  so stale task completion cannot resurrect an active listener.
- Unsupported tunnel lifecycle operations do not bind listeners, create
  destinations, publish LeaseSets, or report running.
- SAM observation is bounded and recoverable after incomplete generations;
  ClientServicesInfo returns a sanitized unavailable result while incomplete.
- RouterInfo preflight rejects unavailable fields and response-budget failures
  before source assembly. Any source or serialization failure returns no
  partial result. Serialized response size is checked after assembly.
- Concurrent readers use immutable snapshots/owner handles; no new lock is
  held across unrelated awaits and no single-owner event receiver is consumed.
- A response lost after commit can cause a client retry to observe the already
  committed generation; this is documented as normal at-most-once persistence
  publication behavior, not silently rolled back.

## 7. Security review

The reviewer searched source, tests, docs, representative serialized fixtures,
and error/tracing paths for passwords, tokens, proxy/outproxy/IRC credentials,
private keys/key files, full private destinations, filesystem roots, temporary
paths, and raw configuration logging.

Conclusions:

- Protected methods require valid authentication; parameter/header conflicts
  fail closed.
- Unsupported tunnel backends open no resources.
- Persistent sensitive data uses restrictive permissions where supported and
  temporary artifacts are cleaned on failure.
- Errors and logs are sanitized; raw secret configuration and private keys are
  not returned or traced.
- Read-only snapshots expose only contract-required public state.
- No owner mutation, event-consumption authority, or task-control authority is
  exposed through observation handles.

## 8. Scope audit

Against baseline `a3c4f469f4877e5ff4a0bb4230da298f0b367ed2`, the prior corrected
production files classify as follows:

| Classification | Files / rationale |
|---|---|
| `i2pcontrol local` | `emissary-cli/src/i2pcontrol/**`, feature wiring in `emissary-cli/Cargo.toml` and `emissary-cli/src/lib.rs` |
| `composition-only seam` | `emissary-cli/src/main.rs`, startup wiring in `emissary-cli/src/i2pcontrol/server.rs` |
| `address-book owner seam` | `emissary-cli/src/address_book.rs` and its control-plane adapter |
| `existing tunnel manager seam` | `emissary-cli/src/tunnel/server.rs` and existing manager wiring |
| `SAM observation seam` | `emissary-core/src/sam/mod.rs`, `session.rs`, and streaming listener modules |
| `bounded RouterInfo owner snapshot` | `emissary-cli/src/i2pcontrol/router_info.rs`, `production.rs`, and handler |
| `unexpected` | none |

M027 itself changes only tests, documentation, and planning records. No broad
router, protocol, crypto, frontend, workflow, release, dependency, or missing
tunnel change was added in this pass.

## 9. Verification commands and outcomes

| Command | Outcome |
|---|---|
| `cargo +nightly fmt --manifest-path emissary-cli/Cargo.toml -- --check` | pass |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | pass — 1,213 tests |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass |
| `cargo check -p emissary-core` | pass |
| `cargo test -p emissary-core` | pass — 1,066 passed, 2 ignored |
| `cargo clippy -p emissary-core --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | fails on pre-existing stable/nightly rustfmt configuration and unrelated workspace formatting; touched CLI package nightly check passes |

Focused evidence also passed:

- `conformance_manifest`: 58 tests;
- compatibility/parser `golden_fixtures`: 44 tests;
- existing `i2pcontrol` integration: 27 tests;
- canonical M027 literal fixtures: 7 tests.

No remote CI, upstream CI, release gate, platform matrix, coverage threshold,
fuzz campaign, network farm, or long soak was required by the plan.

## 10. Unresolved findings and future-plan unblock audit

No unresolved high/medium correctness or security finding remains. The 26
unavailable RouterInfo rows are a documented capability limitation and are the
reason for the selected partial status, not a hidden implementation defect.

There are no registered successor handoffs behind M027. The registry,
implementation README, roadmap, support docs, and source map now agree that
M027 is closed and no future plan is blocked on it. Any future work to add a
RouterInfo owner or a missing tunnel data plane requires a new bounded plan; it
is not silently unblocked or included in M027.

## 11. Internal-only/no-upstream attestation

All external source access was read-only. No upstream or third-party repository
or maintainer channel was mutated. No upstream review, merge, adoption,
submission, solicitation, or contribution artifact was requested or prepared.
All repository writes remained inside `eggstack/emissary`.

M027 is formally closed as `partial Proposal 170 support` against the pinned
2026-05-20 revision. This status does not imply upstream review, acceptance,
certification, adoption, approval, or merge.
