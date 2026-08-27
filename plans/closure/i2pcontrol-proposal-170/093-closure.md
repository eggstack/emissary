# M093 Closure — Post-M092 Independent Tunnel Security Reclosure

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/093-post-m092-tunnel-security-reclosure.md`

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Corrective predecessors:

- M090 server loopback + IRC half-close: `plans/closure/i2pcontrol-proposal-170/090-closure.md`;
- M091 pre-accept stream concurrency (technical history only): `plans/closure/i2pcontrol-proposal-170/091-closure.md`;
- M092 M091 authorization/dependency/containment corrective: `plans/closure/i2pcontrol-proposal-170/092-closure.md`;
- `plans/003-planning-process.md` §§6–8 and §11.

Planning baseline: `944da7b887b6efbd46601e9fad1c853581f40b8e`.
Known valid pre-M091 implementation/closure baseline: `6d631d4423c7faa761b47a84e07436bbaf5d9ad4`.
Reviewed production head: `8860407a79347ce925603821cdb231e47a680623` (the M092-corrected production head audited by M093).
M093 closure/planning commit: `4da022ec874e9915e2d38fe63c609bff537ee8ff`.
Review date: 2026-08-27.

## 1. Disposition

M093 is complete. The independent re-audit of the twelve registered Proposal 170 tunnel backends at the M092-corrected head found no high-, medium-, or low-severity production security/anonymity defect inside the approved Proposal 170 boundary. M090's resolver-free server targets and IRC half-close/drain work remain intact. M091's unauthorized dependency/core/vendor delta is absent. M060/M061/M062 production/dependency containment semantics are restored to their pre-M091 authority. The M088 lower-layer / pre-accept limitation remains the current accepted residual.

M093 inherits no technical production code change beyond a single exact planning-test bookkeeping correction:

- `emissary-cli/tests/m062_dependency_containment.rs::is_authorized_planning_path` gains entries for `plans/closure/i2pcontrol-proposal-170/092-closure.md` and `plans/closure/i2pcontrol-proposal-170/093-closure.md`. The `092-closure.md` entry was an M092 bookkeeping defect (its closure plan/closure allowlist entry was promised by M092 §6 but never landed); the `093-closure.md` entry is the symmetric M093 closure entry this file creates. The change touches the planning-test allowlist only; no production code, dependency, lockfile, core, router, startup, frontend, or Proposal 170 contract path was changed.

M093's reviewed head does not introduce any new production behavior, alternative lower-layer transport, vendoring strategy, parallel SAM stack, broader `emissary-core` hook, replacement dependency mechanism, or change to any of the twelve tunnel backends, the runtime/admission/filter/peer-identity/secret-store owners, the Proposal 170 JSON-RPC contract, or the I2PControl-owned composable runtime.

## 2. Authority and precedence

Per `plans/003-planning-process.md` §6 the authority order is:

1. canonical specification and terminology;
2. accepted ADRs;
3. subsystem roadmap;
4. implementation plan;
5. current repository evidence.

M093 confirms that at the corrected head:

- `plans/000-long-term-specification.md`, `plans/001-terminology-and-domain-model.md`, `plans/002-long-term-roadmap.md`, and `plans/003-planning-process.md` are referenced and unmodified by the M092/M093 lineage.
- ADR-0001, ADR-0002, and ADR-0003 are referenced by the subsystem roadmap and the closure chain; the diff against `6d631d44` (pre-M091 baseline) makes no change to any ADR or to its conclusions.
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` and `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md` are referenced from this closure. Status updates in those roadmaps and `plans/registry.md` made by M093 are bookkeeping only.
- `plans/implementation/i2pcontrol-proposal-170/093-post-m092-tunnel-security-reclosure.md` is the implementation plan that defined this closure.
- Current repository evidence (§§3–8 below) is the underlying authority for the per-invariant status calls.

The M090 closure record remains valid production work and is explicitly retained. The M091 closure record remains technical history only, marked `corrective pass required / superseded by M092` and explicitly disclaimed as current authority. The M092 closure record remains the accepted record of the M091 production/dependency/vendor rollback. M093 builds on those three closures as required by its objective.

## 3. Twelve-tunnel requirement-to-evidence matrix

The twelve registered Proposal 170 tunnel backends were reviewed at `8860407` against the §3 review scope and §5 review requirements. Evidence is classified as: directly verified (D), inherited historical (H), accepted residual (R), unavailable/unsupported (U), or new defect (X).

| # | Tunnel | Type | Primary evidence | Result |
|---|---|---|---|---|
| 1 | `client` | outbound SAM session, bounded local listener, validated destination/port | `backends/client.rs`; runtime `client_listener.rs:121-126` (`publish: false` for outbound-only clients where applicable); `backends/options.rs:89-138` fail-before-allocation option validation | D-pass |
| 2 | `httpclient` | outbound HTTP, I2P-only or explicit configured outproxy, loopback listener validation, bounded request/relay | `backends/http_client.rs`; `backends/filters/http_client.rs`; shared `options` validator; `filters/http.rs` framing | D-pass |
| 3 | `ircclient` | bounded local listener, shared IRC filter with fail-closed DCC/CTCP | `backends/irc_client.rs`; `backends/filters/irc.rs` | D-pass |
| 4 | `socks` | bounded SOCKS parsing, loopback listener, I2P destination or explicit outproxy, local-target rejection | `backends/socks.rs`; `backends/filters/proxy.rs`; shared `options` validator | D-pass |
| 5 | `socksirc` | composition of SOCKS and accepted IRC filter; no unfiltered bypass | `backends/socks_irc.rs` | D-pass |
| 6 | `connectclient` | bounded CONNECT parsing, I2P/outproxy route selection, local-target rejection | `backends/connect_client.rs` | D-pass |
| 7 | `streamrclient` | loopback-only configured UDP target, fixed destination/port tuple, 4095-byte receive buffer, 1200-byte forwarding cap | `backends/streamr.rs:733-782` (`local_loopback_address`, `local_udp_source_allowed`, `payload_is_forwardable`); `streamr.rs:42,44,46` `MAX_TRANSPORT_PACKET`, `MAX_STREAMR_PAYLOAD`, `MAX_SUBSCRIBERS` | D-pass |
| 8 | `server` | accepted-stream raw relay, literal-loopback target, 5s connect bound, 10-min progress-resetting inactivity, half-close drain, no absolute lifetime | `backends/server.rs:28,34,440-565`; M087 invariants directly verified; admission release via `Drop` on `AdmissionLease` | D-pass |
| 9 | `httpserver` | shared accepted-stream identity/admission path; literal-loopback target; fail-closed framing; `Expect` rejected before local connect; spoof/fingerprint stripping; bounded POST accounting keyed by canonical 32-byte ID; finite body/relay deadlines | `backends/http_server.rs:43,46,77-159,438-508,574,730-818`; `backends/filters/http.rs:15-17,179-356`; `backends/runtime/peer_identity_impl.rs:31,80-94` | D-pass |
| 10 | `httpbidirserver` | composite: public server session + fork-local separate unpublished outbound client session | `backends/http_bidir.rs:333-465` (composite spawn); runtime `client_listener.rs:121-126` (`publish: false`); no Java I2P same-manager identity sharing | D-pass |
| 11 | `ircserver` | bounded registration, trusted peer-derived presentation, 5s literal-loopback connect, 10-min progress-resetting post-registration idle, M090 half-close drain, raw post-registration relay | `backends/irc_server.rs:33,37-41,228-234,295-399,401-484,556-624`; M090 invariants directly verified | D-pass |
| 12 | `streamrserver` | loopback-only local UDP, 10 subscribers, 60s expiry, 15s refresh, 4095-byte transport, 1200-byte payload, one-byte control, bounded sequential fanout | `backends/streamr.rs:42-58,274-330,399-472,733-782` | D-pass + R (Sybil monopolization; see §6.7) |

The §3 scope also names the shared runtime/admission/filter/session owners; their state at the corrected head was reviewed in §§4–7 below.

## 4. Security/anonymity review

### 4.1 Server local-target confinement (§5.1)

The three families accept literal loopback spellings before any connection. `httpserver` and inbound `httpbidirserver` use `normalize_loopback_target(&target_host, true)` which returns `Some(IpAddr)` only for `"127.0.0.1"`, `"localhost"`, or `"::1"`; `ircserver` uses the same helper with `allow_ipv6 = false`. The connect sites receive an `IpAddr` and call `TcpStream::connect(SocketAddr::new(target_address, target_port))` — no resolver/NSS/DNS involvement. Non-loopback values are rejected with `BackendError::UnsupportedOption` before any persistent destination lookup. Generic `server` hardcodes `("127.0.0.1", target_port)`. No LAN/clearnet target expansion was introduced.

- `backends/http_server.rs:482-484,512-518,574`
- `backends/http_bidir.rs:520-521`
- `backends/irc_server.rs:332-333,620-624`
- `backends/server.rs:492`

Result: PASS. Direct verification; no defect.

### 4.2 Generic server lifetime and half-close (§5.2)

M087 invariants are intact at the M092-corrected head.

- `GENERIC_SERVER_INACTIVITY = Duration::from_secs(10 * 60)` at `backends/server.rs:34`.
- The `deadline` is reset only by `activity.send_modify(...)` on every successful `relay_direction` write, and each direction owns its own relay task sharing a single `deadline` via the `activity_rx` watch.
- One-sided EOF sets the direction's `active = false`; the other direction continues draining until completion or the inactivity deadline. Tests `generic_server_half_close_drains_the_other_direction` and `generic_server_progress_resets_deadline_without_fixed_lifetime` exercise both halves.
- No absolute maximum stream lifetime is imposed; the inactivity bound is the only time-based bound.
- `TARGET_CONNECT_TIMEOUT = Duration::from_secs(5)` bounds `bounded_target_connect`; failure returns and the admission lease drops with the handler task.

- `backends/server.rs:28,34,491-565,960-988,990-1019,1097-1123`

Result: PASS. M087 directly verified.

### 4.3 IRC server lifecycle (§5.3)

`irc_server.rs` retains bounded registration parsing (`MAX_REGISTRATION_LINES=12`, `MAX_REGISTRATION_LINE=1024`, `REGISTRATION_TIMEOUT=15s`, `REGISTRATION_LINE_TIMEOUT=5s`), trusted peer-derived identity presentation (`USER alice 0 peer.b32.i2p :Alice`), literal-loopback local target with IPv6 disabled, 5-second connect bound, 10-minute progress-resetting inactivity, corrected M090 half-close drain (`remote_eof_allows_local_to_remote_drain`, `local_eof_allows_remote_to_local_drain`, `half_close_completion_releases_admission_lease`), and raw post-registration relay without invented application semantics.

- `backends/irc_server.rs:33,37-41,228-234,295-484,556-624,714-926`

Result: PASS. Direct verification.

### 4.4 HTTP server family (§5.4)

- Identity — sole ingress is `TrustedPeerIdentity::from_stream(&stream)` at `runtime/accepted_server.rs:120`. `peer_identity_impl.rs:31,53-94` enforces bounded Base64 text (≤1024), single `Destination::parse_frame` with empty remainder, canonical full text, and a 32-byte cryptographic ID. `Debug` redacts both fields.
- Framing — ambiguous framing, multiple differing `Content-Length`, `Transfer-Encoding`, `Connection: upgrade`/`Upgrade`, and `Expect` are all rejected in `filters/http.rs:227-356`. `Expect` returns `417 Expectation Failed` from `read_and_sanitize_request` *before* the local-target `TcpStream::connect` at `http_server.rs:482-484`.
- Spoof/fingerprint stripping — `PROXY_IDENTITY`, `I2P_IDENTITY`, and `REQUEST_PRIVACY` headers are stripped; trusted `X-I2P-DestB64`/`X-I2P-DestB32` are replaced from `peer`; `RESPONSE_FINGERPRINTS` and hop-by-hop headers are stripped in the response.
- POST accounting — `PostPeerKey` is `*peer.canonical_id()`; bounded at `MAX_THROTTLE_ENTRIES = 1024` with no eviction of active state.
- Deadlines — `HEADER_TIMEOUT = 15s`, `REQUEST_LINE_TIMEOUT = 5s`, `BODY_TIMEOUT = 60s` for both `copy_body` and `copy_response_body`.
- No speculative body byte cap or fairness redesign was added.

- `backends/http_server.rs:43,46,70-159,438-508,574,730-818,889-902,1136-1187`
- `backends/filters/http.rs:15-17,37-89,227-356,381-406,563-567,735-998`
- `backends/runtime/peer_identity_impl.rs:31,53-130`
- `backends/runtime/admission.rs:283-289`

Result: PASS. Direct verification.

### 4.5 Application admission and pre-accept ordering (§5.5 / §5.6)

The common post-accept `ServerAdmissionState` is authoritative for global concurrency, per-peer concurrency, configured minute/hour/day peer and aggregate rate windows, bounded peer-history/cardinality semantics (`MAX_PEER_ENTRIES = 81920` hard ceiling), transactional admission/release (`Drop for AdmissionLease` decrements global+peer active counts and either schedules expiry or removes the peer record), and bounded `BoundedTaskGroup` handler ownership.

The corrected ordering at `runtime/accepted_server.rs:112-135` is:

```text
remote signed streaming SYN
  → Emissary lower-layer parse/signature/replay (Yosemite crate 0.7.0 via crates.io)
  → Session<style::Stream>::accept()              [accepted_server.rs:115]
  → TrustedPeerIdentity::from_stream(&stream)     [accepted_server.rs:120]
  → ServerAdmissionState::try_acquire(&peer)      [accepted_server.rs:123]
  → handler task ownership (BoundedTaskGroup)     [accepted_server.rs:127-135]
```

The M091 pre-allocation stream-concurrency check is **absent** by design after the M092 rollback; this is the intended M088 boundary, not a defect. Pre-accept signed-SYN/streaming work runs inside Yosemite before `ServerAdmissionState`; that is the accepted lower-layer availability/timing residual, not a direct clearnet identity leak.

- `backends/runtime/accepted_server.rs:5,77-145`
- `backends/runtime/admission.rs:519-671,905-921,1203-1217,1219-1234`
- `backends/runtime/task_group.rs:1-54`
- `Cargo.toml:45` (`yosemite = "0.7.0"` from crates.io; no `path = "vendor/..."` override)

Result: PASS. Direct verification of the post-accept ordering, plus explicit M088 residual for the pre-accept side.

### 4.6 Streamr (§5.7)

All nine Streamr invariants remain in place at the corrected head:

| Invariant | Evidence | Result |
|---|---|---|
| Loopback-only local endpoints | `streamr.rs:59`, `733-778` | D |
| `MAX_SUBSCRIBERS = 10` | `streamr.rs:46` | D |
| `SUBSCRIPTION_EXPIRY = 60s` | `streamr.rs:55` | D |
| `SUBSCRIPTION_REFRESH = 15s` | `streamr.rs:57` | D |
| One-byte control messages | `streamr.rs:293-303` | D |
| `MAX_STREAMR_PAYLOAD = 1200` | `streamr.rs:44,780-782` | D |
| `MAX_TRANSPORT_PACKET = 0xfff` (4095-byte buffer) | `streamr.rs:42` | D |
| Bounded sequential fanout | `streamr.rs:457-468` | D |
| Local-UDP-source policy (`local_udp_source_allowed`) | `streamr.rs:776-778,918-925` | D |

Sybil monopolization of the finite subscriber set is the reference-aligned availability limitation that has been documented since M078 and confirmed by M089; it is retained as an accepted residual. The pre-existing low-severity AGENTS.md / M072 wording discrepancy (`"16 subscribers"` vs the implemented 10) is unchanged from M089 and remains a planning-side reconciliation candidate for a future documentation-only decision — no production impact.

Result: PASS + R (Sybil monopolization only).

### 4.7 `httpbidirserver` (§5.8)

The fork's separate unpublished outbound client session remains isolated from the public server Destination. `http_bidir.rs:333-465` spawns two independent child tasks: a `run_accepted_server` server child with `publish: true` and a `run_client_listener` client child with `publish: false` (suffixed `-client` nickname). The client session has no `DestinationKind::Persistent` and starts with the initial empty destination text `"direct-i2p-only"`. There is no shared session manager or identity reuse; Java I2P's same-manager identity sharing is intentionally not adopted.

- `backends/http_bidir.rs:333-465`
- `backends/runtime/client_listener.rs:121-126`

Result: PASS. Direct verification; no Java-I2P parity change.

### 4.8 Persistent identity and diagnostics (§5.9)

- Backend-owned persistent keys live in `server-destinations/` under the configured state root (`server_secret_store.rs:19,89-96`), bounded to `MAX_STORE_SIZE = 1 MiB` and `MAX_ENTRIES = 1000` (`server_secret_store.rs:22-23,139`).
- Path confinement — `validate_root()` (line 200-225) and `reject_existing_file_link()` / `reject_directory_link()` (publication.rs:184-205) reject symlinks on the store dir, its parent, and any existing file before write. `read_file` (server_secret_store.rs:241) checks both symlink and non-file.
- Restrictive file mode — `set_restrictive_permissions()` (publication.rs:220-231) sets `0o600` on Unix for both async and sync write paths.
- Diagnostic redaction — `StoredDestination`'s `Debug`/`Display` print `"StoredDestination(***)"` / `"***"`; `TrustedPeerIdentity::Debug` prints `<redacted>` for both fields; `AcceptedServerRuntimeError` uses fixed enum variants with no format interpolation; `production.rs:1340` documents "never exposes private key material." The `session_setup_failure_is_sanitized` test (accepted_server.rs:281-304) asserts the failure path contains no private text.
- Crash recovery / atomicity — `publish_with_backup` writes temp, renames current → backup, renames temp → current, syncs directory, and rolls back on rename failure (publication.rs:52-100). `load()` reads current and falls back to backup, reporting corruption as an error (server_secret_store.rs:106-122). The `current_corruption_recovers_valid_backup` and `stale_temp_does_not_override_current` tests exercise both paths.
- Generation-local ephemeral state — `ServerAdmissionState::new()` creates fresh `Arc<AdmissionInner>` with empty peer map and expiry queue; the `restarted_generation_begins_with_empty_rate_and_peer_state` test confirms fresh `(0, 0)` peer and active counts after restart.

- `src/i2pcontrol/server_secret_store.rs:19-386`
- `src/i2pcontrol/production.rs:1340`
- `backends/runtime/accepted_server.rs:281-304`
- `backends/runtime/peer_identity_impl.rs:123-130`
- `backends/runtime/admission.rs:503-517,1203-1217`

Result: PASS. Direct verification across all five sub-invariants.

## 5. Containment review (§6)

`git diff --name-status 6d631d4423c7faa761b47a84e07436bbaf5d9ad4..HEAD`:

| Path | Change | Authority | Result |
|---|---|---|---|
| `plans/closure/i2pcontrol-proposal-170/091-closure.md` | added (M092 disposition amendment) | M092 §6 / M093 bookkeeping | bookkeeping only |
| `plans/closure/i2pcontrol-proposal-170/092-closure.md` | added (M092 closure record) | M092 §6 | bookkeeping only |
| `plans/implementation/i2pcontrol-proposal-170/091-pre-accept-stream-concurrency-boundary-hardening.md` | status/annotation restored to `blocked / superseded by M092` | M092 §6 | bookkeeping only |
| `plans/implementation/i2pcontrol-proposal-170/092-m091-authorization-and-containment-corrective.md` | added (M092 plan) | M092 plan | bookkeeping only |
| `plans/implementation/i2pcontrol-proposal-170/093-post-m092-tunnel-security-reclosure.md` | added (M093 plan) | M093 plan | bookkeeping only |
| `plans/implementation/i2pcontrol-proposal-170/README.md` | status / handoff / sequence updated | M092 §6 / M093 closure | bookkeeping only |
| `plans/registry.md` | status, sequence, ready handoff, recently-closed table, maintenance rules updated | M092 §6 / M093 closure | bookkeeping only |
| `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | status, §1, §7, §8 updated for M092 closed / M093 closed | M092 §6 / M093 closure | bookkeeping only |
| `emissary-cli/tests/m062_dependency_containment.rs` | added `092-closure.md` and `093-closure.md` to `is_authorized_planning_path` | M093 closure | exact planning-test bookkeeping only (no production code) |

Production/dependency diff against `6d631d44`:

- `git diff --stat 6d631d4423c7faa761b47a84e07436bbaf5d9ad4..HEAD -- Cargo.toml Cargo.lock emissary-core/** emissary-cli/src/** plans/implementation/i2pcontrol-proposal-170/061* plans/implementation/i2pcontrol-proposal-170/062*` → empty.
- `git diff --stat 6d631d4423c7faa761b47a84e07436bbaf5d9ad4..HEAD -- emissary-cli/src/i2pcontrol/backends/http_server.rs emissary-cli/src/i2pcontrol/backends/http_bidir.rs emissary-cli/src/i2pcontrol/backends/irc_server.rs` → empty. M090 production delta preserved byte-for-byte.
- `git ls-files | grep '^vendor/yosemite/'` → empty. No vendored Yosemite copy.
- `061-containment-boundary.toml:5,22-26` and `062-dependency-containment.toml:5,19-26,29-35` restored to M090 closure state: `lockfile.expected = "byte-identical to baseline"`, `allowed_production_paths.root_manifests = ["Cargo.toml", "emissary-cli/Cargo.toml"]`, no `Cargo.lock` allowlist entry, `prohibited_production_paths` patterns intact.

Any unexplained core/router/dependency change would block; none is present. The M091 production/dependency/vendor delta is fully absent.

Result: PASS.

## 6. Compatibility and protocol review (§7)

| Item | Status | Evidence |
|---|---|---|
| Proposal 170 JSON-RPC methods/actions/statuses/field spelling | PASS | `src/i2pcontrol/domain/tunnel.rs:30-53` (renames) and no production diff |
| Twelve tunnel type names | PASS | `domain/tunnel.rs:57-70` `ALL_TUNNEL_TYPES` lists exactly: `client`, `httpclient`, `ircclient`, `socks`, `socksirc`, `connectclient`, `streamrclient`, `server`, `httpserver`, `httpbidirserver`, `ircserver`, `streamrserver` |
| Startup-managed tunnel ownership | PASS | `server.rs:1146-1151` test `startup_server_lifecycle_is_rejected_before_store_access`; no ownership model diff |
| RouterInfo 37/1/5 (43 canonical / 37 available / 1 protocol-permitted neutral / 5 unavailable) | PASS | `subsystem/i2pcontrol-proposal-170-roadmap.md:48,88,262,331`; `061-containment-boundary.toml:5`; `062-dependency-containment.toml:5` |
| AddressBook / base-I2PControl unrelated limitations | PASS | No diff against `6d631d44` |
| Public storage format | PASS | `server_secret_store.rs` storage is unchanged |
| Tunnel cryptographic algorithms / path selection / transport | PASS | No diff in `emissary-core/src/crypto/` or `emissary-core/src/transport/`; `server.rs:588-609` `validate_i2cp_options` accepts only `leaseSetEncType` |

Unsupported or underspecified runtime options fail before allocation:

- `backends/options.rs:89-138` `validate_options` rejects `Missing` / `Unsupported` before any runtime/session allocation.
- Per-backend raw-option validation (`http_server.rs:730-818`, `irc_server.rs:644-681`, `server.rs:440-479`, `streamr.rs:784-825`) runs before destination lookup or session creation.
- Streamr rejects non-loopback local addresses at `streamr.rs:733-761` before any UDP socket reservation.

Result: PASS. No contract regression.

## 7. Failure, cancellation, restart, and contention review (§8)

| Family | Admission lease release | Bounded tasks | No lock across I/O | Connect bounded | Half-close / inactivity | Parser bounded | Shutdown teardown |
|---|---|---|---|---|---|---|---|
| Generic `server` | PASS (`AdmissionLease` RAII in handler task) | PASS (`BoundedTaskGroup` + drain) | PASS | PASS (5s) | PASS (M087 directly verified) | N/A | PASS |
| `httpserver` | PASS | PASS | PASS | PASS (5s `CONNECT_TIMEOUT`) | PASS (POST model; bounded `PostLimiter`) | PASS | PASS |
| `httpbidirserver` | PASS (server child shares `BoundedTaskGroup`) | PASS (composite) | PASS | PASS | N/A (composed) | PASS | PASS |
| `ircserver` | PASS | PASS | PASS | PASS (5s) | PASS (M090 directly verified) | PASS | PASS |
| `streamrserver` | N/A (datagram; `MAX_SUBSCRIBERS = 10`) | PASS (`JoinHandle` per tunnel, bounded by `MAX_RUNTIME_TASKS = 1000`) | PASS | N/A (UDP, loopback) | N/A | PASS | PASS |
| Client backends | N/A (no accepted-stream identity) | PASS | PASS | N/A | N/A | PASS | PASS |

Detailed evidence:

- Lease release — `admission.rs:631-671` `Drop for AdmissionLease` decrements global/peer active counts atomically, schedules expiry with history or removes peer record without history. Tests `lease_drop_releases_active_count_exactly_once` and `rejection_does_not_extend_counters_or_expiry` cover 100-iteration acquire/drop correctness.
- Task group — `task_group.rs:1-54` `BoundedTaskGroup` owns `JoinSet` + `Semaphore`; `try_spawn` is permit-gated; `drain(STOP_TIMEOUT)` waits then aborts on timeout. Each `run_accepted_server` invocation constructs fresh state.
- Locks — no production mutex is held across `.await` in `runtime/admission.rs`, `runtime/server.rs`, `backends/server.rs`, `backends/http_server.rs`, `backends/irc_server.rs`, `backends/streamr.rs`. `tokio::sync::Mutex` in `filters/irc.rs:471` is dropped before the I/O write.
- Connect failures — handlers return immediately on connect error and the admission lease drops with the handler; tests `generic_server_idle_expiry_releases_admission_lease`, `http_server` 502 path, and `irc_server.rs:928-938` exercise the bounded-disconnect path.
- Half-close — `server.rs:543-565` and `irc_server.rs:349-396` use the `activity_rx` watch with per-direction `active` flags; tests listed in §4.2/§4.3 cover both directions.
- Parser / peer-state bounds — IRC registration bounded (§4.3); HTTP request framing bounded (§4.4); Streamr `apply_control` rejects on `MAX_SUBSCRIBERS` cap; admission peer map at `MAX_PEER_ENTRIES = 81920`; `POST_THROTTLE_ENTRIES = 1024`.
- Shutdown — `run_accepted_server` breaks the accept loop on cancellation, drains `BoundedTaskGroup(STOP_TIMEOUT = 5s)`. Per-supervisor `stop_generation` paths send cancellation, await with bounded timeout, and abort on timeout. Streamr follows the same pattern.

Result: PASS. Required M098-equivalent `half_close_completion_releases_admission_lease` test exists at `irc_server.rs:900-926` and is passing in the I2PControl CLI green run.

## 8. Application-admission and pre-accept ordering evidence

§4.5 already records the chain at `accepted_server.rs:112-135`. The supporting behavior:

- `TrustedPeerIdentity::from_stream(&stream)` runs after `session.accept()` returns and validates the SAM-reported destination structurally (`peer_identity_impl.rs:53-94`). A malformed remote destination is rejected without invoking the handler and without admitting a peer record (test `malformed_remote_destination_is_rejected_before_handler_invocation`).
- `ServerAdmissionState::try_acquire(&peer)` runs after identity is established and either returns `Allowed(lease)` or `Denied`, with `Denied` paths `continue`-ing without spawning a handler task.
- The handler is invoked only with a `(stream, peer)` tuple where `peer` is a validated `TrustedPeerIdentity` and the `AdmissionLease` is held in the spawned task (drop-on-completion RAII).
- The pre-accept side (signed-SYN/streaming work) runs inside Yosemite 0.7.0 from crates.io, before `session.accept()` returns. That is the M088 residual documented in `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` §5 and accepted by this closure as the current lower-layer disposition.

Result: PASS.

## 9. Verification command outcomes

All required §9 commands were executed against the reviewed head.

| Command | Outcome |
|---|---|
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass; no errors |
| `cargo test -p emissary-core` | pass; 1062 tests, 2 ignored across 5 suites |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | pass; 1696 tests across 24 suites |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m060_containment --test m061_containment --test m062_dependency_containment` | pass; 29 tests across 3 suites (3 + 7 + 19) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment` | pass; 19 tests (after the M093 bookkeeping correction is applied) |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass; no issues |
| `git diff --check` | pass |
| `git ls-files | grep '^vendor/yosemite/'` | empty |

Focused tests:

| Family / concern | Command | Outcome |
|---|---|---|
| Generic server inactivity / half-close | `cargo test -p emissary-cli --no-default-features --features i2pcontrol server::tests` | pass; 146 tests |
| HTTP server (loopback normalization + security filters) | `cargo test -p emissary-cli --no-default-features --features i2pcontrol http_server::tests` | pass; 26 tests |
| HTTP bidirectional (typed-target seam, bounded composite) | `cargo test -p emissary-cli --no-default-features --features i2pcontrol http_bidir::tests` | pass; 12 tests |
| IRC server (registration / connect / inactivity / half-close) | `cargo test -p emissary-cli --no-default-features --features i2pcontrol irc_server::tests` | pass; 28 tests |
| Streamr local boundary / subscriber expiry / fanout | `cargo test -p emissary-cli --no-default-features --features i2pcontrol streamr::tests` | pass; 20 tests |
| HTTP / IRC filters | `cargo test -p emissary-cli --no-default-features --features i2pcontrol filters` | pass; 64 tests |
| Application admission / peer identity | `cargo test -p emissary-cli --no-default-features --features i2pcontrol admission::tests` | pass; 56 tests |
| Persistent server-key store (path / mode / symlink / atomicity) | `cargo test -p emissary-cli --no-default-features --features i2pcontrol server_secret_store::` | pass; 10 tests |
| Shared accepted-server runtime | `cargo test -p emissary-cli --no-default-features --features i2pcontrol runtime::` | pass; 74 tests |
| M060 / M061 / M062 static guards | `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test static_guards --test static_guards_m007` | pass; 58 tests across 2 suites |

`cargo fmt --all -- --check` was attempted but is not green under the installed stable/nightly toolchains due to pre-existing repository/nightly rustfmt configuration drift. The same drift was documented by the M090 and M092 closures; no formatter-only production churn was introduced by M093.

### M090 regression evidence

- `http_server::tests` directly verify M090 literal-loopback target normalization (`http_server.rs:889-902` `target_host_unaffected_by_non_loopback_rejected_at_config` and `expect_request_is_rejected_with_417_before_local_allocation`).
- `irc_server::tests` directly verify M090 literal-loopback target (`irc_server.rs:956-967`) and half-close drain (`irc_server.rs:858-926`).
- The `http_bidir.rs` typed-target seam continues to be exercised by the `http_bidir::tests` suite.

### M092 regression evidence

- M092 production/dependency rollback: production/dependency diff against `6d631d44` is empty.
- Containment: `m060_containment`, `m061_containment`, and `m062_dependency_containment` all green; `061-containment-boundary.toml` and `062-dependency-containment.toml` restored to pre-M091 authority.
- Vendor copy: `vendor/yosemite/**` is absent.

## 10. Independent review rule (§10)

M093 was reviewed independently against the actual M092-corrected production head `8860407a79347ce925603821cdb231e47a680623`. It does not restate M089/M090/M092 closure claims. Each security-critical invariant in §§3–8 above distinguishes:

- **Directly verified behavior** — code/test evidence cited by file:line. Predominant classification throughout this closure.
- **Inherited historical evidence still applicable** — M087 lifetime invariants preserved; M090 loopback/half-close preserved; M078 Streamr bounds preserved; M083 trusted-peer-identity boundaries preserved.
- **Accepted residual limitation** — M088 lower-layer signed-SYN work pre-accept (§4.5); Streamr Sybil monopolization (§4.6); 16-subscriber wording in `AGENTS.md`/old M072 matrix vs implemented 10 (§4.6).
- **Unavailable / unsupported semantic** — none new in M093.
- **New defect** — none. The only M093-introduced change is the exact planning-test allowlist addition.

A code commit, passing compile, or prior closure assertion alone is not treated as closure evidence; this closure cites the file:line for each claim.

## 11. Stop-condition check (§11)

M093 MUST stop and open a new numbered corrective if any of the following is true:

| Stop condition | Found? |
|---|---|
| High- or medium-severity production security/anonymity defect | **No** |
| Unexplained M091 production/dependency artifact remaining after M092 | **No** |
| Containment guard weakened beyond exact planning bookkeeping | **No** |
| Direct identity disclosure path | **No** |
| Unbounded attacker-controlled server state/task growth within the reviewed application boundary | **No** |
| Proposal 170 contract regression | **No** |
| Required fix that would change production code | **No** |

The only correction applied under M093 is `emissary-cli/tests/m062_dependency_containment.rs::is_authorized_planning_path` adding the exact `092-closure.md` and `093-closure.md` planning paths (§1, §12 below). This is exact planning-test bookkeeping; no production behavior changed. Per §11’s permission for low-severity/documentation corrections, it is recorded here and not promoted to a new numbered corrective.

## 12. Low-severity M092 bookkeeping correction applied under M093

The M062 test suite (`emissary-cli/tests/m062_dependency_containment.rs`) runs `allowed_production_paths_match_the_m062_budget` against the diff from `fork_baseline = "a70dd3ac..."` through `HEAD`. The function consults an in-test allowlist that already enumerated all earlier M06x–M091 plan paths and the M092/M093 plan paths but omitted the `092-closure.md` and `093-closure.md` closure paths. The M092 closure record's claim that "all 19 tests pass" was therefore recorded without the allowlist entry the M092 §6 bookkeeping allowance described in registry.md required. After M092 landed at commit `8860407`, `092-closure.md` was added to the working tree, the allowlist was not extended, and the test began failing on every subsequent run.

M093 records this as the one finding of the reclosure, classifies it as:

- **Severity:** low.
- **Nature:** planning-test allowlist bookkeeping defect from M092.
- **Production impact:** none. The test exercises a static allowlist against an exact-path diff glob; the missing entry does not weaken any guard.
- **Fix scope:** exact planning-path entries for `092-closure.md` (carried forward from M092 §6) and `093-closure.md` (this closure).
- **Authorization:** §11 of this plan ("Low-severity/documentation findings may be corrected only if they do not alter production behavior and are explicitly recorded") together with M093 §6's expression that the cumulative M062 planning allowlist may contain "exact M092/M093 plan/closure paths only as planning bookkeeping."

The exact diff:

```text
emissary-cli/tests/m062_dependency_containment.rs:542 (additions)
+            | "plans/closure/i2pcontrol-proposal-170/092-closure.md"
+            | "plans/closure/i2pcontrol-proposal-170/093-closure.md"
```

No other line in `emissary-cli/tests/m062_dependency_containment.rs` was changed by M093. No other test file was changed.

## 13. Future-plan disposition (§3 plan question)

After this closure:

- The tunnel-security line becomes current-head closed. No future implementation handoff is registered, queued, or unblocked; the M093 closure does not name a successor.
- M051 (RouterInfo news/banned-peer source owners) remains independently blocked by its accepted missing sources. The RouterInfo 37/1/5 disposition is unchanged.
- Proposal 170 remains separately partial for accepted source/truthfulness limitations, AddressBook gaps, and base-I2PControl limitations.
- M088's lower-layer / pre-accept limit and Streamr's Sybil-monopolization limit remain accepted residuals; neither authorizes a new implementation handoff.

No corrective pass is opened by M093. No new implementation handoff is registered in the registry.

## 14. Unresolved findings

- **None (high or medium severity).**
- **Low severity (planning only):** the M092 bookkeeping allowlist defect recorded in §12 is corrected under this closure as exact planning-test bookkeeping.
- **Low severity (planning only):** the pre-existing `AGENTS.md` / old M072 capability-matrix wording that says "16 subscribers" while the implementation, tests, and current M078-M089 contract use 10 remains a future documentation-reconciliation candidate; it does not affect runtime behavior and is unchanged from M089.

## 15. Disposition

**M093 is closed.** M093 makes one exact planning-test bookkeeping correction (§12) and no production behavior change. The tunnel-security line is current-head closed at the M092-corrected head. The M087 lifetime invariants, M090 loopback and half-close work, and M078 Streamr bounds remain the operational authority for the application boundary. The M088 lower-layer / pre-accept limit remains the accepted residual.

## 16. Internal-only attestation

All review, evidence gathering, planning, closure, registry, and roadmap writes were confined to the internal `eggstack/emissary` repository. No upstream issue, PR, review, submission, merge request, maintainer contact, contribution artifact, vendored-crate submission to crates.io, or external repository write was opened, drafted, requested, or pushed. External I2P, I2P+, Yosemite, and reference source repositories and specifications remain read-only evidence.
