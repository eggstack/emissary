# M085 Closure — Merged-Head Tunnel Security Reclosure

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/085-merged-head-tunnel-security-reclosure.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`

Corrective predecessors:

- M084: `plans/closure/i2pcontrol-proposal-170/084-closure.md` — merged-head
  integration/planning corrective closed; this closure runs against the
  exactly-recorded post-M084 head.
- M083: `plans/closure/i2pcontrol-proposal-170/083-closure.md` — admission
  capacity / trusted-Destination exactness.
- M082: `plans/closure/i2pcontrol-proposal-170/082-closure.md` — HTTP peer
  identity / `Expect` / POST.
- M081: `plans/closure/i2pcontrol-proposal-170/081-closure.md` — generic server
  `leaseSetEncType` apply-or-reject.
- M080: `plans/closure/i2pcontrol-proposal-170/080-closure.md` — admission
  transactionality/cardinality.
- M078: `plans/closure/i2pcontrol-proposal-170/078-closure.md` — Streamr
  local-boundary hardening.
- M077: `plans/closure/i2pcontrol-proposal-170/077-closure.md` — IRC lifetime
  hardening.
- M079: `plans/closure/i2pcontrol-proposal-170/079-closure.md` — historical
  older-lineage closure only; M085 explicitly supersedes it for current-head
  final certification.

Planning baseline:

- Pre-M084: `e8feb9a3240a5a7b9dd5cc22a4ada47a0d9991ae` (the two-parent merge).
- Post-M084 implementation head: `776407f51e75e0df245a304749b5981e639e9aab`.
- Post-M084 closure head:
  `1196a4d85cecb4f9676a8d87d27c69322816d7a8`.
- M085 post-fix baseline recorded by M084 closure activity:
  `1196a4d85cecb4f9676a8d87d27c69322816d7a8`.

Final M085 reviewed head:

- `a6f18268b8d8724ed826f69614161b5b8d293ef5`
  (subject: `docs(i2pcontrol): record M084 post-fix baseline SHA`).
- Branch: `master`.
- Working tree clean; no uncommitted files; `git diff --check` clean.

M085 is a closure-only milestone and introduces no production change.

## 1. Disposition

M085 closes successfully. The merged-head Proposal 170 tunnel runtime/security
line is independently re-audited against the actual post-M084 repository head
and shows no high or medium security, anonymity, correctness, lifecycle,
option-truthfulness, or containment finding. The full I2PControl test surface,
focused admission/trusted-peer/generic-server/HTTP/IRC/Streamr tests, and the
M061/M062 containment guards pass at the final reviewed head. M085 also does
**not** reopen the unrelated base-I2PControl, AddressBook, M051, or RouterInfo
37/1/5 limitations.

M085 is the **current-head final reclosure authority**. It supersedes M079
only as final certification against the merged composition; M079 remains
historical evidence for the older M077/M078 branch lineage and is retained
without rewriting its existing claim to certify that lineage at its pinned
head. No historical closure was modified. No new dependency, no
`emissary-core/**` production path, no startup/router/frontend refactor, no
Proposal 170 wire/API expansion, no hosted CI/release/fuzz/soak/public-network
machinery, and no upstream interaction was added.

## 2. Why an independent reclosure was required

`master` was formed by merging two separately verified histories:

1. the older M076/M077/M078/M079 tunnel-security lineage — M079 closed the
   integrated head of that lineage at its own pinned baseline
   (`04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`); and
2. the later M080/M081/M082/M083 admission/identity corrective lineage — M083
   closed its implementation lineage and is present in current `master`.

The merging happened in `e8feb9a3`, whose parents are:

- `58a07a2` — older lineage head;
- `569608d` — M083 lineage head.

That merge composition contains a non-trivial intersection that neither
lineage individually audited:

- M083 exact/canonical trusted Destination semantics;
- M083 peer-history / capacity / expiry-index state machine;
- M077 IRC idle/connect behavior consuming the same shared accepted-server
  admission/identity boundary;
- M078 Streamr loopback-only local boundary;
- M081 generic `leaseSetEncType` apply-or-reject;
- M082 HTTP fixed-417 `Expect` rejection and canonical POST key.

Branch-local closure evidence is therefore not sufficient for the merged
composition. M085 re-runs and re-reasons about the actual combined head
rather than copying M079's matrix forward.

M084 had already closed the merge-integration and planning/planning-status
defects described in M085 §3 (IRC admission test fixture, pre-M083 admission
regression redundancy, dropped `is_proxy_identity_header` /
`is_i2p_identity_header` helper bodies, and M062 exact-path bookkeeping
under-coverage). With M084 closed, M085 audits the result of M084 against the
final head rather than against the M079-era tree.

## 3. Merge ancestry and evidence integrity

Verified relevant commits present in the M085 final head (`a6f1826`):

| Milestone | SHA | Subject |
|---|---|---|
| M077 | `0660ca6` | harden IRC server lifetime |
| M078 | `0ff8b22` | harden Streamr local boundary |
| M079 (historical) | `221ad29` | close M079 tunnel security reclosure |
| M080 | `f07bf14` | admission transactionality/cardinality |
| M081 | `cd41d28` | generic server `leaseSetEncType` apply-or-reject |
| M082 | `0fbbe83` | HTTP peer identity / `Expect` framing |
| M083 | `3eaea53` | admission capacity / Destination exactness |
| M084 fix | `776407f` | merged-head integration/planning corrective |
| M084 closure | `84534cb` | close M084 and unblock M085 |
| M084 docs | `1196a4d` | reconcile support status for M084 closure and M085 |
| M085 baseline record | `a6f1826` | record M084 post-fix baseline SHA |

All twelve of the audited production tunnel backends are registered through
the unchanged `TunnelBackendRegistry` (M084 closed with no backend-registry
edits; M085 verifies the registry is still exhaustive and unchanged).

No new `emissary-core/**` production path or dependency was introduced. The
`subtle` ownership invariant from M062 continues to gate `emissary-cli`
feature-disabled builds (the test runs `cargo test --test
m062_dependency_containment` and reports 19 passed).

## 4. Independent review outcomes by area

### 4.1 Trusted peer identity (M085 §7.2)

`emissary-cli/src/i2pcontrol/backends/runtime/peer_identity_impl.rs`:

- production ingress flows through `AcceptedServerConnection::peer`, derived
  from the SAM `STREAM STATUS` remote destination via
  `TrustedPeerIdentity::from_stream` (`accepted_server.rs:120`, `peer_identity_impl.rs:53`);
- textual input is bounded by `MAX_TRUSTED_DESTINATION_B64_TEXT = 1024`
  bytes (`peer_identity_impl.rs:31`) and rejected when empty, oversized,
  whitespace-, or control-containing (`peer_identity_impl.rs:53-69`);
- decoded bytes are consumed with `Destination::parse_frame` and an explicit
  `rest.is_empty()` zero-remainder check (`peer_identity_impl.rs:78-83`);
- downstream B64 text is re-encoded from `parsed.serialize()`, not the
  attacker-supplied input (`peer_identity_impl.rs:91-93`);
- the 32-byte accounting key is `parsed.id()` copied byte-for-byte
  (`peer_identity_impl.rs:85-90`);
- `Debug` redaction covers both `destination` and `canonical_id`
  (`peer_identity_impl.rs:123-131`);
- structurally valid fixtures produced by `test_fixtures::distinct_peer(seed:
  u8)` and `distinct_peer_u32(seed: u32)` vary the public-key prefix bytes
  of `NULL_CERT_DESTINATION_BYTES` (`peer_identity_impl.rs:195-215`) so the
  IRC admission-release regression (line `irc_server.rs:743`) uses a fixture
  that `Destination::parse_frame` accepts with zero remainder and produces
  a unique 32-byte `canonical_id`;
- HTTP request-path identity injection (`filters/http.rs:121-128`) writes
  `X-I2P-DestB64` / `X-I2P-DestB32` from the same `peer` field that the POST
  limiter keys by, so the canonical byte text and the 32-byte accounting
  identity are guaranteed to derive from the same `Destination::parse_frame`
  output.

Evidence (8 focused, no integration):

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol peer_identity
cargo test -p emissary-cli --no-default-features --features i2pcontrol runtime::admission peer_identity
```

Result: 8 passed including:

- `accepts_supported_destinations_and_stores_canonical_text`
- `rejects_one_or_many_trailing_destination_bytes`
- `canonical_id_is_stable_for_the_same_exact_destination`
- `canonical_destination_ids_produce_distinct_32_byte_keys`
- `malformed_destination_text_is_rejected_before_admission`

### 4.2 Admission resource/fairness state (M085 §7.3)

`emissary-cli/src/i2pcontrol/backends/runtime/admission.rs`:

- `ServerAdmissionState::try_acquire` is a single critical section guarded by
  `parking_lot::Mutex<State>` with eight ordered sub-checks — global,
  peer-capacity, peer concurrency, peer rate, aggregate rate (before any
  mutation), and a `try_reserve(1)` for new peers — followed by an atomic
  commit (`admission.rs:519-628`);
- aggregate-rate denial occurs before any peer-state insertion, so
  deny-churn cannot grow peer/expiry state
  (`admission.rs:519-628`, `aggregate_rate_rejection_does_not_create_peer_record`);
- `ServerAdmissionPolicy::required_peer_entries()` is the tightest safe bound
  across all enabled aggregate windows plus a fixed-window overlap term
  (`+1` beyond `ceil(history/window)`) computed with checked arithmetic
  (`admission.rs:139-178`);
- `checked_capacity_math_never_wraps_downward` proves the capacity proof
  returns `None` on overflow rather than wrapping;
- `MAX_PEER_ENTRIES` hard ceiling = `16 MiB / WORST_CASE_BYTES_PER_PEER (200)` =
  81,920; historical policies whose required entries do not fit reject at
  policy construction;
- `peer_history()` is `Some(Duration)` only when minute/hour/day semantics
  are explicitly enabled, and its retention is fixed at the corresponding
  window;
- final-drop lease release
  (`admission.rs:631-715`) either removes the peer record immediately
  (no history, no inactive residue) or inserts exactly one entry into the
  inactive-only `BTreeMap<(Instant, PeerKey), ()>` expiry index (with
  history);
- `State::assert_invariants` (`admission.rs:465-528`) and
  `active_peer_past_expiry_remains_bounded_and_is_reindexed_on_final_drop`
  prove active peers intentionally have no expiry entry, historical inactive
  peers have exactly one, and a final drop restores the authoritative
  registration;
- `BoundedTaskGroup` (used in `accepted_server.rs:110`) makes the active-task
  set bounded by `max_concurrent_connections`.

Evidence (56 focused admission + regression tests):

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol runtime::admission
```

Result: 56 passed including the section 8 adversarial cases:

- `capacity_derivation_accepts_default_and_rejects_unrepresentable_aggregate`
  (rejects `IncoherentCapacity` for day history with unlimited aggregates
  and for huge aggregates over day history);
- `every_historical_peer_window_requires_a_finite_aggregate_bound` (minute,
  hour, and day peer history with all aggregates unlimited all reject
  before runtime state);
- `tightest_aggregate_window_not_field_precedence_controls_capacity` (hour
  and day windows can be tighter than the minute window and govern the
  capacity proof);
- `checked_capacity_math_never_wraps_downward` (overflow returns `None`
  rather than wrapping to a small accepted value);
- `fixed_window_boundary_overlap_is_included_in_capacity_bound` (events
  immediately before and after a fixed-window reset coexist in active state
  and the calculated bound includes both windows plus the concurrency
  margin);
- `no_history_fresh_peer_churn_does_not_accumulate_inactive_state`
  (256 distinct fresh peers under a no-history policy churn with a clean
  `(0, 0)` state at every iteration);
- `aggregate_rate_rejection_does_not_create_peer_record` (denied attempts
  do not mutate the peer map or the expiry index);
- `existing_peer_rate_rejection_does_not_extend_counters_or_expiry` and
  `existing_peer_aggregate_rejection_does_not_extend_counters_or_expiry`
  (denied repeat attempts do not extend counters or add expiry entries);
- `expiry_index_live_entries_remain_bounded_under_repeated_acquire_drop`
  (churn reaps expired entries on the next acquire and keeps active state
  intentionally unindexed);
- `repeated_reap_is_idempotent_for_inactive_history` (repeated reap with
  no new acquire is a no-op);
- `restarted_generation_begins_with_empty_rate_and_peer_state` (a fresh
  generation begins empty and is independent of the prior one);
- `lease_drop_releases_active_count_exactly_once` (100 acquire-drop cycles
  on a single peer converge to zero active count);
- `active_peer_past_expiry_remains_bounded_and_is_reindexed_on_final_drop`
  (active peers that outlive a nominal history deadline stay in the peer
  map until final drop and then reindex correctly).

### 4.3 Generic server and option truthfulness (M085 §7.4)

`emissary-cli/src/i2pcontrol/backends/server.rs`:

- generic `server` always uses `run_accepted_server` (`server.rs:217-236`);
  it does not regress to blind `STREAM FORWARD`;
- `lease_set_enc_type` is validated as the only allowed key in
  `validate_i2cp_options` (`server.rs:493-507`) and threaded into
  `SessionOptions::lease_set_enc_type`
  (`accepted_server.rs:87-95`);
- empty string is normalized to `None` so the SAM wire never receives an
  empty `i2cp.leaseSetEncType=` key (`server.rs:509-517` and
  `lease_set_enc_type_is_threaded_when_present_and_absent_otherwise`);
- non-loopback `TargetHost` is rejected at option validation before
  allocation;
- `validate_raw_options` (`server.rs:429-470`) explicitly allowlists typed
  and recognized raw fields; unrecognized raw keys and recognized-but-unsupported
  fields fail before destination/session/listener allocation;
- `relay_accepted_connection` (`server.rs:480-491`) is an explicit
  `io::copy_bidirectional` from the accepted stream to the fixed loopback
  target — bytes remain byte-transparent;
- persistent identity is owned by `ServerDestinationStore`; `PrivKeyFile`
  input from `rawConfig` is rejected because generic raw configuration is
  not a key-material ingress path.

Evidence (170 tests covering `server`, `runtime::accepted_server`, and the
shared `filters/http.rs` request path used by HTTP):

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol server
```

Result: 170 passed including:

- `accepted_peer_identity_reaches_handler_before_local_target` (handler
  runs only after the trusted peer identity is established; the local
  target check is unused because the test handler does not connect it,
  but the same code path is exercised by HTTP/IRC handlers which **do**
  connect it);
- `malformed_remote_destination_is_rejected_before_handler_invocation`
  (the legacy placeholder text `"peer-destination"` is rejected without
  invoking the handler or admitting a peer record);
- `session_setup_failure_is_sanitized` (`SessionSetup` errors do not leak
  the private key in `Debug`/display);
- `lease_set_enc_type_is_threaded_when_present_and_absent_otherwise`
  (the validated `leaseSetEncType` value is passed through `runtime_config`
  and the empty string becomes `None`);
- `non_loopback_target_host_is_rejected` and the family of generic option
  capability rejections.

### 4.4 HTTP/httpbidir boundary (M085 §7.5)

`emissary-cli/src/i2pcontrol/backends/filters/http.rs`:

- every incoming `X-I2P-*`, `X-Forwarded-*`, and `I2P_IDENTITY` /
  `PROXY_IDENTITY` header is removed via `is_i2p_identity_header` /
  `is_proxy_identity_header` (`filters/http.rs:37-81` and
  `filters/http.rs:421-428`) before the request reaches the local target;
- `read_and_sanitize_request` (`filters/http.rs:179-300`) refuses
  duplicate `Host`, applies host rewrite from `HttpServerPolicy`, and
  injects the **exact** peer-derived
  `X-I2P-DestB64` / `X-I2P-DestB32` whose underlying bytes are the
  authenticated `Destination::parse_frame` output (`filters/http.rs:294-298`);
- `Expect` rejection (`filters/http.rs:243-247`) returns
  `RequestSanitizeError::ExpectUnsupported` mapped to a fixed
  `417 Expectation Failed` response **before** any `TcpStream::connect` to
  the local target;
- request framing rejects duplicate/conflicting `Content-Length`,
  `Transfer-Encoding`, and obsolete folded-header lines;
- `read_and_filter_response` (`filters/http.rs:302-369`) strips every
  entry in `RESPONSE_FINGERPRINTS` (Date, Server, Alt-Svc, Age, Via,
  X-Cache, X-Powered-By, hop-by-hop, provider/cache/trace/adopted
  fingerprints, etc.) (`filters/http.rs:25-71`) and re-validates framing,
  but does not rewrite the application body;
- `PostLimiter` keys on `PostPeerKey([u8; 32])` derived from the same
  canonical 32-byte Destination ID (`http_server.rs:68-82`);
- `MAX_THROTTLE_ENTRIES = 1024` (`http_server.rs:44`) and the table is
  `HashMap` plus a fixed-window red-black style index; denial of unseen
  peers when full **never evicts active state**
  (`http_server.rs:130-150`);
- `httpbidirserver` consumes the **same** `make_accepted_handler` from
  `http_server.rs:437` (`http_bidir.rs:333-336`) and shares `PostLimiter`
  through the same instantiation; `persisted_public_destination_does_not_select_local_target`
  in `http_server.rs:893` and `http_bidir.rs:940` proves hosting-destination
  metadata cannot become a local target;
- `HostingDestination` is server identity metadata only and is not a
  local-target selector.

Evidence (121 focused HTTP tests):

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol http
```

Result: 121 passed including:

- `expect_request_is_rejected_with_417_before_local_allocation`
  (`http_server.rs:1103`) — the local backend listener is bound, the
  test deliberately avoids accepting on it for 150 ms, then asserts the
  accept did **not** time out by reading the timeout error back; the
  peer only ever receives `HTTP/1.1 417 Expectation Failed\r\n` and
  `Connection: close`;
- `end_to_end_path_sanitizes_before_local_connect_and_filters_response`
  (`http_server.rs:1043`) — attacker-injected
  `Host: evil.i2p` / `x-i2p-destb64: attacker` are stripped, only the
  configured `Host: configured.i2p` reaches the local backend, and the
  response `Server: hidden` is stripped before relay;
- `post_limiter_is_bounded_and_peer_keyed` /
  `post_limiter_keys_distinct_peers_independently` /
  `post_limiter_denies_churn_without_evicting_active_entries` /
  `post_limiter_counts_only_write_methods` (`http_server.rs:939-992`)
  prove the 32-byte canonical keying, churn-safe denial, and that
  a fresh identity cannot reset an already-active throttle;
- `rejected_post_does_not_connect_to_local_backend`
  (`http_server.rs:994`) — the local listener accept times out and the
  test asserts the timeout `Err` rather than a successful connect;
- `persisted_public_destination_does_not_select_local_target`
  (`http_server.rs:893` and `http_bidir.rs:940`) — even with
  `hosting_destination = Some("published-server-destination")`, the
  resulting `target_host` is `127.0.0.1` and never the published
  metadata.

### 4.5 IRC boundary (M085 §7.6)

`emissary-cli/src/i2pcontrol/backends/irc_server.rs`:

- `read_registration` (`irc_server.rs:401-466`) is bounded to
  `MAX_REGISTRATION_LINES = 12` per-line, with a per-line timeout and a
  total `REGISTRATION_TIMEOUT = 15s`; only `CAP` / `PASS` /
  `AUTHENTICATE` / `PING` / `PONG` / `NICK` (one parameter) / `USER`
  (≥4 parameters) are accepted;
- `looks_like_wrong_protocol` (`irc_server.rs:468-484`) returns true for
  obvious HTTP/DHT probes and causes the registration reader to fail
  closed;
- `rewrite_server_user` (`irc_server.rs:448-466`) replaces the
  hostname with the SAM-derived `peer.b32.i2p` so the local target
  receives an unambiguous, peer-derived hostname;
- `connect_local_target` is bounded by `TARGET_CONNECT_TIMEOUT = 5s`
  (`irc_server.rs:31, 326-340`);
- `relay_with_inactivity` (`irc_server.rs:343-380`) is a bidirectional
  `io::copy_bidirectional`-equivalent relay whose idle expiry
  (`POST_REGISTRATION_INACTIVITY = 10 minutes`) is **reset by any
  successful traffic in either direction** (`irc_server.rs:366-373`);
  there is **no** fixed total lifetime for active sessions;
- the shared `ServerAdmissionState` and `try_acquire` call
  (`irc_server.rs:295-323`) sit **before** the local target connect; the
  lease is moved into the relay task so success, error, EOF, idle
  expiry, panic isolation, cancellation, abort, and stop each release
  the lease exactly once;
- `MAX_REGISTRATION_LINE = 1024` keeps each line bounded;
- `tests` consume `test_fixtures::distinct_peer(7)` rather than the
  removed arbitrary-string `TrustedPeerIdentity::for_test` helper
  (`irc_server.rs:743`), so the structural validity invariant
  established in M083 is preserved on the M077/M083 shared boundary.

Evidence (22 focused IRC server tests):

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol irc_server
```

Result: 22 passed including:

- `registration_rewrites_trusted_peer_and_rejects_http`
  (`irc_server.rs:696`) — `USER alice 0 spoofed-host :Alice` is rewritten
  to `USER alice 0 peer.b32.i2p :Alice`; a `GET / HTTP/1.1` probe fails;
- `registration_bounds_are_enforced`
  (`irc_server.rs:721`) — a 12-line flood of `CAP LS` fails closed;
- `registered_idle_peer_expires_and_releases_admission`
  (`irc_server.rs:735`) — advancing paused time by
  `POST_REGISTRATION_INACTIVITY` returns from the relay with `Ok(())`,
  and a fresh `try_acquire` for the same peer succeeds because the
  lease was released by the relay task;
- `activity_resets_idle_deadline_without_fixed_lifetime`
  (`irc_server.rs:765`) — three round-trips each spaced at
  `POST_REGISTRATION_INACTIVITY − 60s` (≈ 9 minutes) reach 27 minutes of
  paused time without the relay finishing, plus a further 2-minute
  advance (29 minutes total) before the relay is finally aborted; this
  proves the "active traffic survives beyond twenty minutes of paused
  time" requirement;
- `traffic_in_either_direction_resets_idle_deadline`
  (`irc_server.rs:795`) — a local→remote `PING` payload resets the
  idle deadline the same way a remote→local payload does;
- `inactivity_closes_both_relay_directions`
  (`irc_server.rs:822`) — after `POST_REGISTRATION_INACTIVITY` paused
  time, both relay directions return a close error and subsequent
  `write_all` from either side fails;
- `remote_eof_ends_relay` and additional `relay_with_inactivity`
  directed tests.

### 4.6 Streamr boundary (M085 §7.7)

`emissary-cli/src/i2pcontrol/backends/streamr.rs`:

- `local_loopback_address` (`streamr.rs:733-755`) rejects non-loopback
  values for `TargetHost` / `Host` / `ReachableBy` /
  typed `ListenInterface` **before** session or socket allocation;
  `DEFAULT_BIND_ADDRESS = 127.0.0.1` (`streamr.rs:59`);
- `local_udp_source_allowed` (`streamr.rs:776-786`) drops observed
  non-loopback UDP sources as defense in depth;
- `SubscriptionState::apply_control` (`streamr.rs:282-308`) accepts
  exactly one-byte `0` (subscribe/refresh) and `1` (unsubscribe); longer
  or empty controls, and any non-loopback/whitespace/slash/control-char
  peer text over `MAX_STREAMR_DESTINATION_TEXT = 524` bytes
  (`streamr.rs:53`) are ignored; the subscriber set never grows past
  `MAX_SUBSCRIBERS = 10` because new subscriptions only succeed if the
  peer is already present **or** `entries.len() < MAX_SUBSCRIBERS`;
- `MAX_TRANSPORT_PACKET = 0xfff = 4095` and `MAX_STREAMR_PAYLOAD = 1200`
  are bound constants (`streamr.rs:42, 44`);
- `SUBSCRIPTION_EXPIRY = 60s` and `SUBSCRIPTION_REFRESH = 15s`
  (`streamr.rs:55, 57`) match the proposal reference;
- the server fanout (`streamr.rs:457-478`) snapshots the subscriber list
  before sending, then sequentially writes to each subscriber — no
  per-packet unbounded task queue and no lock across network I/O;
- the client (`streamr.rs:333-396`) refreshes every 15 s, attempts one
  unsubscribe on a 100 ms shutdown, and forwards payloads only to the
  fixed configured loopback UDP target; the producer address is never
  read from remote input;
- on restart, the `server-destinations/` store preserves the persisted
  private identity; subscriber state is intentionally ephemeral and is
  cleared on restart (`streamr.rs:457-481` and the codebase-wide
  boundary that no subscriber state is persisted).

Evidence (20 focused Streamr tests):

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol streamr
```

Result: 20 passed including:

- `subscriptions_are_bounded_and_refresh_in_place`
  (`streamr.rs:874`) — exactly `MAX_SUBSCRIBERS` subscribe; the 11th
  subscribe from a different peer is rejected and the existing ten
  entries survive unchanged; a refresh `peer-0` at `+30s` keeps the
  count at `MAX_SUBSCRIBERS` and updates in place;
- `invalid_controls_and_expiry_do_not_leak_state`
  (`streamr.rs:887`) — empty/two-byte/non-matching-byte controls,
  peer-name overflow, whitespace, and slash-bearing names are rejected
  without creating state; a round-trip subscribe/unsubscribe leaves the
  set empty; expiry removes entries older than 60 s;
- `destination_text_bound_matches_reference_representation`
  (`streamr.rs:905`);
- `local_udp_source_policy_is_loopback_only`
  (`streamr.rs:917`) — `127.0.0.1` / `[::1]` sources are accepted,
  public-IPv4/IPv6 sources are rejected;
- `payload_and_transport_bounds_remain_exact`
  (`streamr.rs:927`) — `1200` accepts, `1201` rejects, transport cap
  is `4095`, expiry is 60 s, refresh is 15 s;
- `loopback_defaults_and_explicit_v4_v6_addresses_are_accepted`
  (`streamr.rs:954`) and `non_loopback_addresses_reject_before_runtime_reservation`
  (`streamr.rs:1007`) — non-loopback `TargetHost`, `ListenInterface`,
  `Host`, and `ReachableBy` all produce
  `BackendError::UnsupportedOption` before any session/listener is
  reserved; `inspect` reports `TunnelRuntimeState::Stopped` for those
  definitions;
- `local_target_is_fixed_by_configuration`
  (`streamr.rs:1071`) — even with the producer address set, the client
  output target is the fixed configured `127.0.0.1:9000` — the remote
  destination is **not** equal to the local target string.

### 4.7 Lifecycle and generation ownership (M085 §7.8)

Every real backend in the M085 final head:

- validates typed/raw options before store lookup, listener, session,
  socket, or task allocation — all of which return
  `BackendError::UnsupportedOption` / `BackendError::MissingOption` /
  `BackendError::Internal` rather than silently persisting a stored
  configuration;
- publishes readiness only after the underlying session and local owner
  are established — `AcceptedServerRuntimeConfig::ready`
  `oneshot::Sender<Result<String, _>>` is sent only after
  `Session::<style::Stream>::new` returns `Ok` (`accepted_server.rs:82-107`);
- stop is idempotent (the `BoundedTaskGroup` is drained on cancellation
  and the watcher returns the same final state regardless of repeat
  calls; `irc_server.rs:270-277`, `server.rs:267-275`,
  `http_server.rs:388-394`, `http_bidir.rs:...`,
  `streamr.rs:565-575`);
- restart is full stop followed by a new generation
  (`server.rs:137-176`, `http_server.rs:248-303`);
- tasks carry a `generation` (`u64`) and stale tasks cannot mutate
  replacement entries because `mark_running` /
  `publish_destination` / `complete` check the generation before
  mutating and silently no-op otherwise;
- persistent secret/public identity ownership remains backend-owned
  (`ServerDestinationStore`) and is redacted from `Debug`/`Display`
  output (`tunnel_manager.rs` exports redaction logic); private-key
  raw input via `PrivKeyFile` from `rawConfig` is rejected;
- ephemeral admission / POST / subscriber / task state does not cross
  generations
  (`restarted_generation_begins_with_empty_rate_and_peer_state` for
  admission; identical expectations for POST, subscribers, and tasks);
- one failed `StartOnLoad` definition does not fail unrelated
  definitions — `production_composition.rs` reconciles
  per-definition and isolates individual failures.

Evidence is consolidated in:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol m033_tunnel_lifecycle
cargo test -p emissary-cli --no-default-features --features i2pcontrol static_guards
```

Both suites pass under the full I2PControl target (`cargo test ... i2pcontrol`
records 1674 passed across 24 suites; M033 lifecycle and static-guard suite
counts are part of that aggregate).

### 4.8 Containment and dependency review (M085 §7.9)

M085 was deliberately a closure-only pass. Production changes are limited to
`emissary-cli/src/i2pcontrol/**` per the M084 fix landed at `776407f` and
matching status reconciliation. No new `emissary-core/**` production path,
no startup/router/frontend refactor, no new dependency, no default-feature
widening was introduced.

`subtle` is owned by `emissary-cli` as an **optional** dependency and only
activates under `--features i2pcontrol`; this ownership is locked by
`m062_dependency_containment.rs::subtle_feature_ownership_is_preserved`,
which runs `LocalFeatureGraph::transitive_closure` over both the workspace
`Cargo.toml` and the resolved feature graph to demonstrate that the
`subtle` crate is reachable only from `i2pcontrol`. Tokio `test-util` is in
`[dev-dependencies]` only and never affects the default or feature-disabled
production builds. The lockfile is byte-identical to the M084 post-fix
baseline (the same `m062_dependency_containment.rs` test asserts
`lockfile_is_byte_identical_to_fork_baseline`).

Final containment evidence:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment           → 7 passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment → 19 passed
```

M061 covers source-path ownership and the M076/M079 prefix-match helpers
in `filters/http.rs`. M062 covers dependency ownership and the exact-path
planning-bookkeeping list extended by M084 to include the merged
M077/M078/M079 closure paths and the M084/M085 planning paths; the
relevant entries are:

```text
plans/closure/i2pcontrol-proposal-170/077-closure.md
plans/closure/i2pcontrol-proposal-170/078-closure.md
plans/closure/i2pcontrol-proposal-170/079-closure.md
plans/implementation/i2pcontrol-proposal-170/084-merged-head-integration-and-planning-corrective.md
plans/implementation/i2pcontrol-proposal-170/085-merged-head-tunnel-security-reclosure.md
plans/closure/i2pcontrol-proposal-170/084-closure.md
plans/closure/i2pcontrol-proposal-170/085-closure.md
```

The new closure path (`plans/closure/i2pcontrol-proposal-170/085-closure.md`)
is added in M085 only via this closure-document creation; no other M085 file
touches a production path.

## 5. Option-capability matrix disposition (M085 §10)

The integrated twelve-type matrix is preserved from M079 and re-evidenced
against the M085 final head. Every runtime-relevant option has exactly one
disposition: applied, rejected, or unsupported-but-allowed-then-rejected.
`docs/i2pcontrol/tunnel-backends.md` records the matrix unchanged.

| Tunnel type | Applied / runtime-owned fields | Recognized but not implemented — rejected before allocation |
|---|---|---|
| `client` | TargetDestination, TargetPort, ListenInterface, ListenPort | access / plaintext / custom / I2CP and other typed / raw fields |
| `httpclient` | listener + proxy auth, HTTP policy, direct I2P target, explicit I2P outproxy | TLS, arbitrary clearnet / direct target, unsupported proxy / outproxy modes, custom / I2CP |
| `ircclient` | I2P target, ports, listener, common IRC filter | IRC automation, access / auth / WEBIRC / cloak, custom / I2CP |
| `socks`, `socksirc` | loopback + authenticated listener, SOCKS CONNECT policy and (for `socksirc`) IRC filter | BIND, UDP ASSOCIATE, arbitrary DNS, unsafe targets, custom / I2CP |
| `connectclient` | listener / auth, strict CONNECT parsing, direct I2P or explicit I2P outproxy | unsupported methods, unsafe direct targets, unsupported proxy / outproxy modes, custom / I2CP |
| `streamrclient` | producer destination, loopback target, UDP target / source ports, 15-second refresh | non-loopback addresses, tunnel shaping / signature / encryption, custom / I2CP |
| `server` | loopback target / port, persistent identity, shared admission and `leaseSetEncType` | access / privacy / consumer / signature / hashcash, unsupported raw fields, custom / I2CP |
| `httpserver` | loopback target, Host / access policy, shared admission, peer-keyed POST limiter, persistent identity | TLS, proxy / outproxy, `FilterFilePath`, `UniqueLocalAddressPerClient`, `MultiHoming`, underspecified periods / ban time, custom / I2CP |
| `httpbidirserver` | shared filtered inbound HTTP path plus authenticated local proxy, loopback bind / target, shared admission and POST limiter | unsupported TLS / outproxy / filter / address / period options, custom / I2CP |
| `ircserver` | bounded registration, trusted peer hostname, loopback target, shared admission, inactivity relay | IRC automation, WEBIRC / cloak / access / auth / DCC options, custom / I2CP |
| `streamrserver` | persistent identity, loopback UDP source, ten subscribers, 60-second expiry, 1200-byte payload, bounded transport | non-loopback addresses, tunnel shaping / signature / encryption, custom / I2CP |

`MaxConcurrentConns`, `ClientPer{Minute,Hour,Day}`, `TotalInPer{Minute,Hour,Day}`,
`PostLimit`, `PostLimitTime`, `FilterFilePath`, `UniqueLocalAddressPerClient`,
`MultiHoming`, `PerClientPeriod`, `TotalPeriod`, `TotalBanTime`, access-list
fields, `leaseSetEncType` and declared `LeaseSet*` / session options, raw
`i2cp` / `i2p.tunnel.customOptions`, and the local host / interface fields
are all consumed in the matching backend or rejected before allocation.

`HostingDestination` is published server identity metadata and is not a
local target selector. Persist-and-ignore does not occur anywhere in the
audited code. The matrix above is unchanged from M079 evidence and is
reproduced here against the M085 final head with no field disposition
altered by M084 or any other post-M079 commit.

## 6. Changed-path summary (M085)

The only paths M085 itself touched are planning and documentation:

| Path | M085 change |
|---|---|
| `plans/closure/i2pcontrol-proposal-170/085-closure.md` | created by M085 closure |
| `plans/registry.md` | status reconciliation for M085 closure |
| `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | status reconciled for M085 closure |
| `plans/implementation/i2pcontrol-proposal-170/README.md` | implementation handoff README reconciled |
| `docs/i2pcontrol/proposal-170-support.md` | status line updated |
| `docs/i2pcontrol/tunnel-manager.md` | status line updated |
| `docs/i2pcontrol/tunnel-backends.md` | status line updated |

No `emissary-cli/src/i2pcontrol/**` production file was edited by M085. No
`emissary-core/**` file was edited by M085. No Cargo dependency or feature
was changed by M085.

The pre-M085 head is already reproducible; `git rev-parse HEAD` reports
`a6f18268b8d8724ed826f69614161b5b8d293ef5` for the reviewed head. The
pre-M085 closure baseline named in `plans/registry.md` and
`i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` is
`1196a4d85cecb4f9676a8d87d27c69322816d7a8`. M085 itself adds only this
closure record and the consequential status-document edits.

## 7. Verification commands and outcomes

```text
cargo check -p emissary-cli --no-default-features                                                       → clean; 0 crates compiled
cargo check -p emissary-cli --no-default-features --features i2pcontrol                                  → clean
cargo check -p emissary-core                                                                             → clean; 0 crates compiled
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings   → no issues
cargo test -p emissary-cli --no-default-features --features i2pcontrol                                     → 1674 passed across 24 suites
cargo test -p emissary-cli --no-default-features --features i2pcontrol irc_server                          → 22 passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol runtime::admission                  → 56 passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol peer_identity                       → 8 passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol server                             → 170 passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol http                               → 121 passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol streamr                            → 20 passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment            → 7 passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment → 19 passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment → 26 passed across 2 suites
git diff --check                                                                                            → clean
```

Per the AGENTS.md note, `cargo fmt --all -- --check` is run on the
repository-accepted stable form. The repository-wide command cannot honor
`rustfmt.toml` `imports_granularity = "Crate"`, `comment_width = 100`,
`trailing_comma = "Vertical"`, etc. because those options require nightly.
Stable Rust reports inherited drift outside any path M085 edited. M085
files outside source (`*.md`, `*.toml`) are not subject to rustfmt. The
scoped nightly rustfmt check on the four touched paths was used by the M084
closure; M085 does not author any new Rust source file, so no new scoped
nightly rustfmt run is required.

## 8. Unresolved findings

| Severity | Finding | Disposition |
|---|---|---|
| low / environmental | Repository-wide stable `cargo fmt --all -- --check` cannot honor `rustfmt.toml` nightly-only options and reports inherited drift outside M085's touched files | Documented limitation also reported in M083 / M084 closures; scoped nightly rustfmt on the four touched Rust files (from M084) was clean; no runtime action required |
| low | M085 is a closure-only audit; it does not change production behavior | By design and explicitly authorized by M085 §6 non-goals; no residual finding |

No high or medium security, anonymity, correctness, lifecycle,
option-truthfulness, or containment finding remains in M085 scope.

## 9. Disposition for downstream plans

M085 closes successfully. The Proposal 170 tunnel runtime/security line is
complete against the pinned Proposal 170 `2026-05-20` revision and the
current internal fork head. No future implementation plan in this
workstream remains blocked by the merged-head audit.

Specifically:

- The `I2PControl Proposal 170 tunnel security hardening` subsystem
  roadmap transitions from `corrective pass required` to **closed**.
  M085 is the final corrective in the sequence; no M086 or later
  security-corrective plan is registered or prewritten. The
  `current tunnel-security sequence` table in `plans/registry.md`
  records the closed disposition for every milestone M074-M085.
- The `I2PControl Proposal 170 tunnel runtime completion` subsystem
  roadmap remains independently closed from its own history
  (`plans/closure/i2pcontrol-proposal-170/072-closure.md` and M073-M084).
  M085 does not reopen or modify it.
- The `I2PControl Proposal 170 source / truthfulness` roadmap remains
  independently `partial Proposal 170 support` because M051 and the
  RouterInfo 37/1/5 disposition are out of M085 scope.

Per the AGENTS.md / `plans/003-planning-process.md` rule that only the
**next** dependency-ready implementation plan is registered `ready`, no new
implementation plan is required here. After M085 closes, `plans/registry.md`
records no active dependency-ready implementation plans under the
tunnel-security subsystem. A new dependency-ready plan, if needed in the
future, would have to be added by a separate planning process and is **not**
created by this closure.

## 10. Documentation reconciliation

Reconciled so that all active support surfaces agree that the
tunnel-security reclosure is complete:

| Document | Reconciled statement |
|---|---|
| `plans/registry.md` | M085 closed; tunnel-security hardening roadmap closed; no future security handoff is blocked |
| `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | status transitions from `corrective pass required` to `closed`; M085 is the final milestone |
| `plans/implementation/i2pcontrol-proposal-170/README.md` | M084 closed and M085 closed; tunnel runtime/security line is complete against the pinned contract and current internal fork head |
| `docs/i2pcontrol/proposal-170-support.md` | M080-M085 closed; tunnel runtime/security line is complete |
| `docs/i2pcontrol/tunnel-manager.md` | M080-M085 server security corrective sequence closed; M085 final reclosure accepted |
| `docs/i2pcontrol/tunnel-backends.md` | M080-M085 server security corrective sequence closed; M085 final reclosure accepted; integrated twelve-type runtime/security line complete |
| `docs/i2pcontrol/streamr-runtime.md` | unchanged; the runtime boundary remains bounded datagram under M071 + M078 |

The reorganization is documentation-only. No history was rewritten: M079 is
retained as historical older-lineage evidence; M083 / M084 are listed as
controlling for the current head. The unrelated Proposal 170 limitations
(RouterInfo 37/1/5, M051, AddressBook, base I2PControl gaps) remain
documented in their respective subsystems and are explicitly out of M085
scope.

## 11. Internal-only external-interaction attestation

External specifications and pinned local dependency source were used only as
read-only behavioral evidence throughout the workstream. No upstream
repository, issue, pull request, review, merge request, discussion,
maintainer channel, submission, contribution artifact, adoption request, or
fork-merge activity was opened, drafted, mutated, or requested. No
hosted CI, release machinery, fuzz farm, soak harness, benchmark gate, or
public-network deanonymization experiment was introduced. All planning,
implementation, documentation, commit, and push activity is limited to the
authorized internal `eggstack/emissary` repository.

## 12. Final status

- M085: **closed**.
- The Proposal 170 **tunnel runtime/security line** is complete against
  the pinned Proposal 170 `2026-05-20` revision and the current internal
  fork head.
- The separately accepted partial Proposal 170 source / truthfulness
  state, RouterInfo 37/1/5 disposition, M051 blocker, AddressBook
  limitations, and unrelated base I2PControl limitations remain
  unchanged and explicitly out of M085 scope.
- No upstream review, acceptance, merge, adoption, submission, or
  contribution artifact is implied or authorized.

## M086 erratum — `MAX_PEER_ENTRIES` capacity arithmetic

The M085 closure text above transcribed the capacity calculation as
`16 MiB / 200 = 81,920`. That number is incorrect. M086 corrects the closure
record without changing its chronology or disposition:

```text
HARD_PEER_STATE_MEMORY_BUDGET = 16 * 1024 * 1024 = 16,777,216 bytes
WORST_CASE_BYTES_PER_PEER = 200
MAX_PEER_ENTRIES = 16,777,216 / 200 = 83,886 (integer division)
```

The authoritative Rust constant/expression is `(16 * 1024 * 1024) / 200` and
does not change. The former `81,920` text was a closure-document arithmetic
error only; it did not affect policy construction, runtime behavior, or any
test M085 executed. This explicit erratum is supplied by M086 rather than
making the original M085 record appear to have contained the corrected value.
